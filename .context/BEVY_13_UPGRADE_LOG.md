# Bevy 0.13 Upgrade Log

## Changes Made

### 1. Dependencies Update
- Updated `bevy` to version "0.13.0"
- Updated `bevy_egui` to version "0.25.0"

### 2. Breaking Changes Fixed

#### Window Configuration
- Removed deprecated `fit_canvas_to_parent` field
- Updated window configuration to use new Bevy 0.13 API

#### Input System
- Updated `Input` type imports from `bevy::input`
- Changed `KeyCode` usage to match new input system

#### Material System
- Removed `TypeUuid` and `uuid` attributes from material structs
- Updated material implementations to match new Bevy 0.13 API

#### Shape System
- Replaced deprecated `Quad` with `Rectangle` primitive
- Updated mesh creation code to use new shape system

### 3. Shader Updates
- Changed binding groups from `@group(1)` to `@group(0)`
- Ensured sequential binding indices (0 through 4)
- Added `globals` binding at index 5
- Standardized uniform variable names across shaders
- Fixed storage class declarations to use proper uniform types

### 4. Entity Management
- Improved entity cleanup in window resize handler
- Added existence checks before despawning entities
- Separated entity despawning and spawning logic
- Added safety checks in entity management code

## Attempted Solutions That Didn't Work

1. **Initial Entity Management**
   - First attempt: Simple entity replacement without checks
   - Issue: Caused panics when trying to despawn non-existent entities
   - Reason: Entities could be despawned by other systems

2. **Shader Binding Approach 1**
   - Attempted to keep `@group(1)` and adjust pipeline
   - Issue: Shader validation errors
   - Reason: Pipeline layout didn't match shader expectations

3. **Material System Update**
   - Tried keeping `TypeUuid` with different configuration
   - Issue: Compilation errors
   - Reason: Feature completely removed in Bevy 0.13

4. **Window Resize Handler**
   - Initial attempt: Update entities in place
   - Issue: Entity state inconsistencies
   - Reason: Race conditions between despawn and spawn

## Current State

### Working Features
- Basic visualization rendering
- Material system integration
- Window resize handling
- Shader compilation and validation

### Known Issues
- Occasional entity management panics during window resize
- Need to verify shader performance with new binding setup
- Potential race conditions in entity cleanup

## Next Steps

1. Consider implementing a more robust entity management system
2. Add better error handling for shader validation
3. Implement proper cleanup on visualization type changes
4. Add logging for better debugging
5. Consider adding entity existence verification in more places

## Performance Considerations

- New shader binding system may need optimization
- Entity cleanup could be batched for better performance
- Consider implementing entity pools for frequent spawning/despawning
