use crate::{
    helper_types::{Diff, Frame},
    input_stream::InputStream,
    motion_input::MotionInput,
};

use bevy::{prelude::*, utils::HashMap};

use types::{LRDirection, MoveId, StickPosition};

/// This is a component and used as an interface
/// Main tells this what Actions to send what events from
#[derive(Debug, Default, Component)]
pub struct InputParser {
    events: Vec<MoveId>,

    registered_inputs: HashMap<MoveId, MotionInput>,
    head: Frame,
    relative_stick: StickPosition,
}
impl InputParser {
    pub fn load(inputs: HashMap<MoveId, &str>) -> Self {
        Self {
            registered_inputs: inputs
                .into_iter()
                .map(|(id, definition)| (id, definition.into()))
                .collect(),
            ..Default::default()
        }
    }

    pub fn register_input(&mut self, id: MoveId, input: MotionInput) {
        self.registered_inputs.insert(id, input);
    }

    pub fn get_absolute_stick_position(&self) -> StickPosition {
        self.head.stick_position
    }

    pub fn get_relative_stick_position(&self) -> StickPosition {
        self.relative_stick
    }

    pub fn drain_events(&mut self) -> Vec<MoveId> {
        self.events.drain(..).collect()
    }

    pub fn clear_head(&self) -> bool {
        self.head.stick_position == StickPosition::Neutral && self.head.pressed.is_empty()
    }

    fn add_frame(&mut self, diff: Diff, facing: &LRDirection) {
        self.head.apply(diff.clone());

        self.relative_stick = facing.mirror_stick(self.head.stick_position);
        let relative_diff = Diff {
            stick_move: diff.stick_move.map(|stick| facing.mirror_stick(stick)),
            ..diff
        };

        self.parse_inputs(&relative_diff);
    }

    fn parse_inputs(&mut self, diff: &Diff) {
        let frame = Frame {
            stick_position: self.relative_stick,
            ..self.head.clone()
        };

        self.events
            .extend(self.registered_inputs.iter_mut().filter_map(|(id, input)| {
                input.advance(diff, &frame);
                if input.is_done() {
                    input.clear();
                    return Some(*id);
                }
                None
            }));
    }
}

pub fn parse_input<T: InputStream + Component>(
    mut characters: Query<(&mut InputParser, &mut T, &LRDirection)>,
) {
    for (mut parser, mut reader, facing) in characters.iter_mut() {
        if let Some(diff) = reader.read() {
            parser.add_frame(diff, facing);
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use time::sleep;
    use types::GameButton;

    use crate::helper_types::InputEvent;
    use crate::testing::TestInputBundle;
    use crate::testing::TestStream;

    const TEST_MOVE: MoveId = 1;
    const SECOND_TEST_MOVE: MoveId = 2;

    use crate::{CHARGE_TIME, MAX_SECONDS_BETWEEN_SUBSEQUENT_MOTIONS};

    use super::*;

    #[test]
    fn hadouken_recognized() {
        let mut interface = TestInterface::with_input("236f");

        interface.add_stick_and_tick(StickPosition::S);
        interface.add_stick_and_tick(StickPosition::SE);
        interface.add_stick_and_tick(StickPosition::E);
        interface.assert_no_events();

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn inputs_expire() {
        let mut interface = TestInterface::with_input("236f");

        interface.add_stick_and_tick(StickPosition::S);
        interface.add_stick_and_tick(StickPosition::SE);
        interface.add_stick_and_tick(StickPosition::E);
        interface.assert_no_events();

        interface.sleep(MAX_SECONDS_BETWEEN_SUBSEQUENT_MOTIONS);

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_no_events();
    }

    #[test]
    fn sonic_boom_recognized() {
        let mut interface = TestInterface::with_input("c46f");

        interface.add_stick_and_tick(StickPosition::W);
        interface.sleep(CHARGE_TIME);

        interface.add_stick_and_tick(StickPosition::E);
        interface.assert_no_events();

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn sonic_boom_needs_charge() {
        let mut interface = TestInterface::with_input("c46f");

        interface.add_stick_and_tick(StickPosition::W);
        interface.tick();

        interface.add_stick_and_tick(StickPosition::E);
        interface.assert_no_events();

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_no_events();
    }

    #[test]
    fn normal_recognized() {
        let mut interface = TestInterface::with_input("f");

        interface.assert_no_events();
        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn command_normal_recognized() {
        let mut interface = TestInterface::with_input("2f");

        interface.add_stick_and_tick(StickPosition::S);
        interface.assert_no_events();
        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn slow_command_normal_recognized() {
        let mut interface = TestInterface::with_input("2f");

        interface.add_stick_and_tick(StickPosition::S);
        interface.assert_no_events();

        interface.sleep(MAX_SECONDS_BETWEEN_SUBSEQUENT_MOTIONS);

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn multibutton_normal_recognized() {
        let mut interface = TestInterface::with_input("[fs]");

        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_no_events();
        interface.add_button_and_tick(GameButton::Strong);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn multibutton_normal_recognized_despite_order() {
        let mut interface = TestInterface::with_input("[fs]");

        interface.add_button_and_tick(GameButton::Strong);
        interface.assert_no_events();
        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_test_event_is_present();
    }

    #[test]
    fn multiple_events() {
        let mut interface = TestInterface::with_inputs("2f", "f");

        interface.add_stick_and_tick(StickPosition::S);
        interface.assert_no_events();
        interface.add_button_and_tick(GameButton::Fast);
        interface.assert_both_test_events_are_present();
    }

    struct TestInterface {
        world: World,
        stage: SystemStage,
    }
    impl TestInterface {
        fn with_input(input: &str) -> TestInterface {
            TestInterface::new(vec![(TEST_MOVE, input)])
        }

        fn with_inputs(input: &str, second_input: &str) -> TestInterface {
            TestInterface::new(vec![(TEST_MOVE, input), (SECOND_TEST_MOVE, second_input)])
        }

        fn new(moves: Vec<(MoveId, &str)>) -> TestInterface {
            let mut world = World::default();

            let mut stage = SystemStage::parallel();
            stage.add_system(parse_input::<TestStream>);

            world
                .spawn()
                .insert_bundle(TestInputBundle::new(moves.into_iter().collect()))
                .insert(LRDirection::Right);

            let mut tester = TestInterface { world, stage };
            tester.tick();

            tester
        }

        fn tick(&mut self) {
            self.stage.run(&mut self.world);
        }

        fn add_button_and_tick(&mut self, button: GameButton) {
            self.add_input(InputEvent::Press(button));
            self.tick();
        }

        fn add_stick_and_tick(&mut self, stick: StickPosition) {
            self.add_input(InputEvent::Point(stick));
            self.tick();
        }

        fn add_input(&mut self, change: InputEvent) {
            for mut reader in self
                .world
                .query::<&mut TestStream>()
                .iter_mut(&mut self.world)
            {
                reader.push(change.clone());
            }
        }

        fn sleep(&mut self, seconds: f32) {
            sleep(Duration::from_secs_f32(seconds + 0.1));
            self.tick();
        }

        fn assert_test_event_is_present(&mut self) {
            self.assert_event_is_present(TEST_MOVE);
        }

        fn assert_both_test_events_are_present(&mut self) {
            self.assert_event_is_present(TEST_MOVE);
            self.assert_event_is_present(SECOND_TEST_MOVE);
        }

        fn assert_event_is_present(&mut self, id: MoveId) {
            let parser = self
                .world
                .query::<&InputParser>()
                .iter(&self.world)
                .next()
                .unwrap();

            assert!(parser.events.contains(&id));
        }

        fn assert_no_events(&mut self) {
            let parser = self
                .world
                .query::<&InputParser>()
                .iter(&self.world)
                .next()
                .unwrap();

            assert!(
                parser.events.is_empty(),
                "Expected no events, found {:?}",
                parser.events,
            );
        }
    }
}
