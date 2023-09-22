pub enum Error {
    /// Fetcher can't fetch a data in this moment
    FetcherNotAvailable,
    /// Post isn't found
    PostNotFound,
    /// An error on fetching of post's content
    PostContentNotProvided,
    /// Tags of a post isn't found
    TagsNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Fetcher {
    type PostID;
    type Post;

    async fn all_posts(limit: usize, offset: usize) -> Result<Vec<Self::Post>>;
    async fn post_by_id(id: Self::PostID) -> Result<Self::Post>;
}

#[async_trait]
pub trait Post {
    type Tag;
    type ID;

    async fn title() -> Result<String>;
    async fn content() -> Result<String>;
    async fn id() -> Result<Self::ID>;
    async fn tags() -> Result<Vec<Self::Tag>>;
}
