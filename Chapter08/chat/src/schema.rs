table! {
    channels (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Text,
        is_public -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    memberships (id) {
        id -> Int4,
        channel_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    messages (id) {
        id -> Int4,
        timestamp -> Timestamp,
        channel_id -> Int4,
        user_id -> Int4,
        text -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Text,
    }
}

joinable!(channels -> users (user_id));
joinable!(memberships -> channels (channel_id));
joinable!(memberships -> users (user_id));
joinable!(messages -> channels (channel_id));
joinable!(messages -> users (user_id));

allow_tables_to_appear_in_same_query!(
    channels,
    memberships,
    messages,
    users,
);
