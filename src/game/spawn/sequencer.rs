//! Spawn the sequencer.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    game::{
        assets::{FontKey, HandleMap, SfxKey},
        audio::sfx::PlaySfx,
    },
    screen::Screen,
    ui::{
        interaction::{InteractionPalette, InteractionQuery},
        palette::{
            ACTIVE_BEAT_BUTTON, HOVERED_ACTIVE_BEAT_BUTTON, HOVERED_INACTIVE_BEAT_BUTTON,
            INACTIVE_BEAT_BUTTON, PLAYING_ACTIVE_BEAT_BUTTON, PLAYING_INACTIVE_BEAT_BUTTON,
        },
        widgets::Widgets,
    },
    AppSet,
};

pub const NUM_SYNTH_NOTES: usize = 8;
pub const NUM_BEATS_IN_SEQUENCE: usize = 32;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_sequencer);
    app.observe(play_sequence);
    app.observe(pause_sequence);
    app.observe(reset_sequence);
    app.observe(play_beat);
    app.register_type::<Sequencer>();
    app.register_type::<GameAction>();
    app.register_type::<SequencerAction>();
    app.insert_resource(Sequence::new());
    app.insert_resource(SequenceState::new());
    app.add_systems(Update, handle_game_action.run_if(in_state(Screen::Playing)));
    app.add_systems(
        Update,
        (
            handle_sequencer_action.run_if(in_state(Screen::Playing)),
            update_sequence_timer.in_set(AppSet::TickTimers),
        ),
    );
}

#[derive(Event, Debug)]
pub struct SpawnSequencer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Sequencer;

/// The current sequence, ordered by beats. If a row appears in the set for a given beat, then that instrument is active on that beat.
#[derive(Resource)]
pub struct Sequence(Vec<HashSet<SequencerRow>>);

impl Sequence {
    /// Creates a sequence with all the notes off
    fn new() -> Sequence {
        Sequence((0..NUM_BEATS_IN_SEQUENCE).map(|_| HashSet::new()).collect())
    }
}

fn spawn_sequencer(
    _trigger: Trigger<SpawnSequencer>,
    mut commands: Commands,
    font_handles: Res<HandleMap<FontKey>>,
) {
    commands
        .spawn((
            Name::new("Sequencer UI Root"),
            Sequencer,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    bottom: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            },
        ))
        .with_children(|children| {
            spawn_controls(children, &font_handles);
            spawn_synth_section(children, &font_handles);
            spawn_percussion_section(children, &font_handles);
        });
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum GameAction {
    Play,
    Pause,
    Stop,
}

fn handle_game_action(mut button_query: InteractionQuery<&GameAction>, mut commands: Commands) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                GameAction::Play => commands.trigger(PlaySequence),
                GameAction::Pause => commands.trigger(PauseSequence),
                GameAction::Stop => commands.trigger(ResetSequence),
            }
        }
    }
}

#[derive(Resource)]
pub struct SequenceState {
    beat_timer: Timer,
    beat: usize,
}

impl SequenceState {
    fn new() -> SequenceState {
        let mut beat_timer = Timer::from_seconds(0.15, TimerMode::Repeating);
        beat_timer.pause();
        SequenceState {
            beat_timer,
            beat: 0,
        }
    }
}

/// Event that starts the sequence playing
#[derive(Event)]
struct PlaySequence;

fn play_sequence(
    _: Trigger<PlaySequence>,
    mut sequence_state: ResMut<SequenceState>,
    mut commands: Commands,
) {
    if sequence_state.beat_timer.elapsed().is_zero() {
        commands.trigger(PlayBeat(0));
    }
    sequence_state.beat_timer.unpause();
}

/// Event that stops the sequence and without resetting it to the beginning
#[derive(Event)]
struct PauseSequence;

fn pause_sequence(_: Trigger<PauseSequence>, mut sequence_state: ResMut<SequenceState>) {
    sequence_state.beat_timer.pause();
}

/// Event that stops the sequence and resets it to the beginning
#[derive(Event)]
struct ResetSequence;

fn reset_sequence(
    _: Trigger<ResetSequence>,
    mut sequence_state: ResMut<SequenceState>,
    mut button_query: Query<(&InteractionPalette, &mut BackgroundColor), With<BeatButton>>,
) {
    sequence_state.beat = 0;
    sequence_state.beat_timer.pause();
    sequence_state.beat_timer.reset();

    for (palette, mut background_color) in button_query.iter_mut() {
        *background_color = BackgroundColor(palette.none);
    }
}

/// Event that plays all the active notes on a single beat
#[derive(Event)]
struct PlayBeat(usize);

fn update_sequence_timer(
    time: Res<Time>,
    mut sequence_state: ResMut<SequenceState>,
    mut commands: Commands,
) {
    sequence_state.beat_timer.tick(time.delta());
    if sequence_state.beat_timer.just_finished() {
        sequence_state.beat = (sequence_state.beat + 1) % NUM_BEATS_IN_SEQUENCE;
        commands.trigger(PlayBeat(sequence_state.beat))
    }
}

fn play_beat(
    trigger: Trigger<PlayBeat>,
    sequence: Res<Sequence>,
    mut button_query: Query<(&BeatButton, &InteractionPalette, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    let beat = trigger.event().0;
    for row in &sequence.0[beat] {
        commands.trigger(PlaySfx(row.to_sfx_key()));
    }

    for (button, palette, mut background_color) in button_query.iter_mut() {
        if button.beat == beat {
            if button.active {
                *background_color = BackgroundColor(PLAYING_ACTIVE_BEAT_BUTTON);
            } else {
                *background_color = BackgroundColor(PLAYING_INACTIVE_BEAT_BUTTON);
            }
        } else {
            *background_color = BackgroundColor(palette.none);
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum SequencerAction {
    ToggleBeat,
}

fn handle_sequencer_action(
    mut button_query: InteractionQuery<(
        &SequencerAction,
        &mut InteractionPalette,
        &mut BeatButton,
    )>,
    mut sequence: ResMut<Sequence>,
    mut commands: Commands,
) {
    for (interaction, (action, mut palette, mut beat_button)) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                SequencerAction::ToggleBeat => {
                    println!("toggling beat button {beat_button:?}"); //TODO
                    beat_button.toggle();
                    if beat_button.active {
                        sequence.0[beat_button.beat].insert(beat_button.row);
                        commands.trigger(PlaySfx(beat_button.row.to_sfx_key()));
                        palette.none = ACTIVE_BEAT_BUTTON;
                        palette.hovered = HOVERED_ACTIVE_BEAT_BUTTON;
                        palette.pressed = INACTIVE_BEAT_BUTTON;
                    } else {
                        sequence.0[beat_button.beat].remove(&beat_button.row);
                        palette.none = INACTIVE_BEAT_BUTTON;
                        palette.hovered = HOVERED_INACTIVE_BEAT_BUTTON;
                        palette.pressed = ACTIVE_BEAT_BUTTON;
                    }
                }
            }
        }
    }
}

fn spawn_controls(parent: &mut ChildBuilder, font_handles: &HandleMap<FontKey>) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                top: Val::Px(0.0),
                justify_self: JustifySelf::Start,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(5.0),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ..default()
        })
        .with_children(|children| {
            // play button
            children
                .small_button(">", font_handles)
                .insert(GameAction::Play);

            // pause button
            children
                .small_button("||", font_handles)
                .insert(GameAction::Pause);

            // stop button
            children
                .small_button("[]", font_handles)
                .insert(GameAction::Stop);
        });
}

fn spawn_synth_section(parent: &mut ChildBuilder, font_handles: &HandleMap<FontKey>) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                justify_self: JustifySelf::Start,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ..default()
        })
        .with_children(|children| {
            for i in (0..NUM_SYNTH_NOTES).rev() {
                spawn_sequencer_row(children, SequencerRow::SynthNote(i), font_handles);
            }
        });
}

fn spawn_percussion_section(parent: &mut ChildBuilder, font_handles: &HandleMap<FontKey>) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                justify_self: JustifySelf::Start,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            ..default()
        })
        .with_children(|children| {
            spawn_sequencer_row(children, SequencerRow::HiHat, font_handles);
            spawn_sequencer_row(children, SequencerRow::Snare, font_handles);
            spawn_sequencer_row(children, SequencerRow::Kick, font_handles);
        });
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum SequencerRow {
    SynthNote(usize),
    HiHat,
    Snare,
    Kick,
}

impl SequencerRow {
    /// Gets the sfx corresponding to this row
    fn to_sfx_key(self) -> SfxKey {
        match self {
            SequencerRow::SynthNote(x) => SfxKey::Synth(x),
            SequencerRow::HiHat => SfxKey::HiHat,
            SequencerRow::Snare => SfxKey::Snare,
            SequencerRow::Kick => SfxKey::Kick,
        }
    }
}

impl std::fmt::Display for SequencerRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequencerRow::SynthNote(i) => format!("Note {i}").fmt(f),
            SequencerRow::HiHat => "Hi-hat".fmt(f),
            SequencerRow::Snare => "Snare".fmt(f),
            SequencerRow::Kick => "Kick".fmt(f),
        }
    }
}

#[derive(Component, PartialEq, Eq, Debug)]
pub struct BeatButton {
    row: SequencerRow,
    beat: usize,
    active: bool,
}

impl BeatButton {
    /// Toggles whether a note will be played on this beat or not
    fn toggle(&mut self) {
        self.active = !self.active;
    }
}

fn spawn_sequencer_row(
    parent: &mut ChildBuilder,
    row: SequencerRow,
    font_handles: &HandleMap<FontKey>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                justify_self: JustifySelf::Start,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(3.0),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        })
        .with_children(|children| {
            children.label(row.to_string(), font_handles);
            for i in 0..NUM_BEATS_IN_SEQUENCE {
                children.spawn((
                    Name::new("Button"),
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(30.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(INACTIVE_BEAT_BUTTON),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    InteractionPalette {
                        none: INACTIVE_BEAT_BUTTON,
                        hovered: HOVERED_INACTIVE_BEAT_BUTTON,
                        pressed: ACTIVE_BEAT_BUTTON,
                    },
                    SequencerAction::ToggleBeat,
                    BeatButton {
                        row,
                        beat: i,
                        active: false,
                    },
                ));
            }
        });
}
