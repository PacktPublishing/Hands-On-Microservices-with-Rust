use diesel::{self, prelude::*};

mod schema {
    table! {
        comments {
            id -> Nullable<Integer>,
            uid -> Text,
            text -> Text,
        }
    }
}

use self::schema::comments;
use self::schema::comments::dsl::{comments as all_comments};

#[table_name="comments"]
#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
pub struct Comment {
    pub id: Option<i32>,
    pub uid: String,
    pub text: String,
}

#[derive(FromForm)]
pub struct NewComment {
    pub uid: String,
    pub text: String,
}

impl Comment {
    pub fn all(conn: &PgConnection) -> Vec<Comment> {
        all_comments.order(comments::id.desc()).load::<Comment>(conn).unwrap()
    }

    pub fn insert(comment: NewComment, conn: &PgConnection) -> bool {
        let t = Comment { id: None, uid: comment.uid, text: comment.text };
        diesel::insert_into(comments::table).values(&t).execute(conn).is_ok()
    }
}
