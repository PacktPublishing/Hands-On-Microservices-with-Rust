table! {
    users {
        id -> Integer,
        email -> Text,
    }
}

table! {
    channels {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
        is_public -> Bool,
    }
}

table! {
    memberships {
        id -> Integer,
        channel_id -> Integer,
        user_id -> Integer,
    }
}

table! {
    messages {
        id -> Integer,
        timestamp -> Timestamp,
        channel_id -> Integer,
        user_id -> Integer,
        text -> Text,
    }
}
