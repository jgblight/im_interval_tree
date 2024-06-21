#[cfg(not(feature = "arc"))]
pub type Shared<T> = std::rc::Rc<T>;

#[cfg(feature = "arc")]
pub type Shared<T> = std::sync::Arc<T>;
