//! Spawn the sequencer.

use bevy::prelude::*;

use crate::{
    game::assets::{FontKey, HandleMap},
    screen::Screen,
    ui::{
        interaction::{InteractionPalette, InteractionQuery},
        palette::{
            ACTIVE_BEAT_BUTTON, HOVERED_ACTIVE_BEAT_BUTTON, HOVERED_INACTIVE_BEAT_BUTTON,
            INACTIVE_BEAT_BUTTON,
        },
        widgets::Widgets,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_sequencer);
    app.register_type::<Sequencer>();
    app.register_type::<GameAction>();
    app.register_type::<SequencerAction>();
    app.add_systems(Update, handle_game_action.run_if(in_state(Screen::Playing)));
    app.add_systems(
        Update,
        handle_sequencer_action.run_if(in_state(Screen::Playing)),
    );
}

#[derive(Event, Debug)]
pub struct SpawnSequencer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Sequencer;

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

fn handle_game_action(mut button_query: InteractionQuery<&GameAction>) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                GameAction::Play => println!("play pressed"),   //TODO
                GameAction::Pause => println!("pause pressed"), //TODO
                GameAction::Stop => println!("stop pressed"),   //TODO
            }
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
) {
    for (interaction, (action, mut palette, mut beat_button)) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                SequencerAction::ToggleBeat => {
                    println!("toggling beat button {beat_button:?}"); //TODO
                    beat_button.toggle();
                    if beat_button.active {
                        palette.none = ACTIVE_BEAT_BUTTON;
                        palette.hovered = HOVERED_ACTIVE_BEAT_BUTTON;
                        palette.pressed = INACTIVE_BEAT_BUTTON;
                    } else {
                        palette.none = INACTIVE_BEAT_BUTTON;
                        palette.hovered = HOVERED_INACTIVE_BEAT_BUTTON;
                        palette.pressed = ACTIVE_BEAT_BUTTON;
                    }
                }
            }
        }
    }
}

const NUM_SYNTH_NOTES: u32 = 8;
const NUM_BEATS_IN_SEQUENCE: u32 = 32;

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
            for i in 0..NUM_SYNTH_NOTES {
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SequencerRow {
    SynthNote(u32),
    HiHat,
    Snare,
    Kick,
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
    beat: u32,
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
