use super::*;
use crate::discord::{FriendStatus, RelationshipInfo};

const BLOCKED_USER: u64 = 50;
const IGNORED_USER: u64 = 60;
const OTHER_USER: u64 = 70;

fn state_with_relationship_authors(display_options: DisplayOptions) -> DashboardState {
    let guild_id = Id::new(1);
    let channel_id: Id<ChannelMarker> = Id::new(2);
    let mut state = DashboardState::new_with_display_options(display_options);
    state.push_event(guild_create_event(
        guild_id,
        "guild",
        vec![text_channel_info(guild_id, channel_id, "general")],
    ));
    state.confirm_selected_guild();
    state.confirm_selected_channel();
    state.push_event(AppEvent::RelationshipsLoaded {
        relationships: vec![
            RelationshipInfo::test(Id::new(BLOCKED_USER), FriendStatus::Blocked),
            RelationshipInfo {
                ignored: true,
                ..RelationshipInfo::test(Id::new(IGNORED_USER), FriendStatus::Friend)
            },
        ],
    });
    for (message_id, author_id) in [(10, OTHER_USER), (11, BLOCKED_USER), (12, IGNORED_USER)] {
        state.push_event(message_create_event(
            guild_text_message(message_id, format!("msg {message_id}"))
                .with_author_id(Id::new(author_id)),
        ));
    }
    state.push_event(latest_history_loaded(channel_id, Vec::new()));
    state
}

fn message_ids(state: &DashboardState) -> Vec<u64> {
    state
        .messages()
        .into_iter()
        .map(|message| message.id.get())
        .collect()
}

#[test]
fn blocked_and_ignored_messages_are_shown_by_default() {
    let state = state_with_relationship_authors(DisplayOptions::default());

    assert_eq!(message_ids(&state), vec![10, 11, 12]);
}

#[test]
fn hide_blocked_messages_only_hides_blocked_authors() {
    let state = state_with_relationship_authors(DisplayOptions {
        hide_blocked_messages: true,
        ..DisplayOptions::default()
    });

    assert_eq!(message_ids(&state), vec![10, 12]);
}

#[test]
fn hide_ignored_messages_only_hides_ignored_authors() {
    let state = state_with_relationship_authors(DisplayOptions {
        hide_ignored_messages: true,
        ..DisplayOptions::default()
    });

    assert_eq!(message_ids(&state), vec![10, 11]);
}

#[test]
fn toggling_hide_options_from_display_popup_updates_messages() {
    let mut state = state_with_relationship_authors(DisplayOptions::default());
    state.open_options_popup();
    let labels: Vec<_> = state
        .display_option_items()
        .iter()
        .map(|item| item.label)
        .collect();
    let blocked_index = labels
        .iter()
        .position(|label| *label == "Hide blocked users' messages")
        .expect("blocked option row should exist");
    let ignored_index = labels
        .iter()
        .position(|label| *label == "Hide ignored users' messages")
        .expect("ignored option row should exist");

    for _ in 0..blocked_index {
        state.move_option_down();
    }
    state.toggle_selected_display_option();
    assert!(state.display_options().hide_blocked_messages);
    assert_eq!(message_ids(&state), vec![10, 12]);

    for _ in blocked_index..ignored_index {
        state.move_option_down();
    }
    state.toggle_selected_display_option();
    assert!(state.display_options().hide_ignored_messages);
    assert_eq!(message_ids(&state), vec![10]);
}
