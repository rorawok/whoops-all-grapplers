- `FlowControl` enum and `Action` enum.
	- Action is something tangible that happens in the game
	- Flow control is an abstract meta-level representation of decisions and timings
	- Move phases vec is `Vec<FlowControl>` 
- Actions
	- Start animation
	- Start move
	- Spawn a hitbox
	- Spawn a projectile
	- Grab
	- Impulse
	- Force (impulse over time)
	- Special
		- For [[Gi of the old masters]] and [[Stance system]]
		- Generic 'other' options
- Flow control
	- Wait
		- Fn that determines wait duration
			- Takes in current situation
		- Actions don't have a duration, they are only concerned with what to do when they happen.
		- Has a cancellable bool, cancel level is derived from the move type
	- Action
		- Fn that determines what comes out
			- Takes in current situation
		- Better over branches for minor adjustments and many permutations.
- Other adjustments
	- Animation is an action, get rid in `Move`
	- `Branch` is `DynamicAction` now
	- `Move` requirements should be a function that takes in the Situation, get rid of  `Requirements`, as it's cumbersome to maintain


`FlowControl` `impl`s:
- `From<Fn (Situation) -> Action>` -> `FlowControl::Action`
- `From<Action>` -> `FlowControl::Action`
	- This is the same as `|_: Situation| Action`
- `From<Fn (Situation) -> (usize, bool)>` -> `FlowControl::Wait`
- `From<(usize, bool)>` -> `FlowControl::Wait`
	- This is the same as `|_: Situation| (usize, bool)`
