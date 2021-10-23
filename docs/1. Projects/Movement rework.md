# Physics
- [ ] Impulse (velocity change)
	- Fire and forget, these cannot be accessed afterwards
- [ ] Continuous force
	- Acceleration: f32
		- How much it influences speed
	- Maximum speed: Optional f32
		- If None, apply always
		- If Some, apply if current speed is less than listed limit
	- Minimum speed: Optional f32
		- If None, apply always
		- If Some, apply if current speed is more than listed limit
- [ ] Re-calculate total velocity each tick
	- Start with previous velocity
	- Continuous force logic
		- Collect ones that want to be applied
		- Modify velocity (don't apply anything before checking everything)
	- Add impulses to velocity
	- Clamp velocity between zero and max speed
		- Make sure near zero movement stops
	- If colliding to a movable target
		- Half velocity in that direction
		- Apply velocity to other physicsobject
	- If colliding to an immovable target
		- Zero all velocity in that direction
- [ ] Detect collisions
	- Set a value in physicsobject
	- Collision data:
		- Which direction is colliding
			- Round to eight directional
		- Is other collider movable or not
- [ ] Interface
	- [ ] Add, remove, or update a continuous force
		- (Dash, Run, Crawl, Drag, Gravity)
	- [ ] Add an impulse
	- [ ] Get current velocity

# Basic ground movement
- [ ] forward or back = run
	- [ ] Apply continuous force
- [ ] directly down = ducking (used to dodge stuff)
	- [ ] Shrink visuals
	- [ ] Shrink hitbox
- [ ] down forward or back = crawl
	- [ ] Shrink visuals (but less than ducking)
	- [ ] Shrink hitbox (but less than ducking)
	- [ ] Apply continuous force

# Basic air movement
- [ ] Up = neutral jump
	- [ ] Apply impulse
	- [ ] Start and end diagonal grace period
		- Grace period is maybe 5 frames
- [ ] Up diagonal within diagonal grace period changes it to a diagonal jump
	- Velocity:
		- Y: Diagonal jump y - (neutral jump y - current y)
			- This is to compensate for already spent jump velocity
		- X: Diagonal jump x

# Dashing
- [ ] 656 or 454 to dash
- [ ] Define dash in terms of travel time and distance like jumping
- [ ] Dashing state (Uninterruptable start, interruptable follow up)
	- In this state, continuous movement is applied according to a function
		- Start fast, constant slide, drag to a stop
	- Dash start
		- Character is busy
		- Character is invulnerable
	- Dash end
		- Can jump and do attacks
		- Can't run or crawl
		- Can maybe duck
		- Can't block