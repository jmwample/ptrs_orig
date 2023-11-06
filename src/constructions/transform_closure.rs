
//! A stream transform that is created from a closure that will be called continuously to transform data.
//! this is a uni-directional transform, so you will need two separate transforms to create a duplex
//! stream pluggable transport.
//!
//! This is currently in limbo because async closures are not yet supported in rust, which makes
//! this significantly harder to work around. For now this is an unnecessary feature that will be
//! more of a distraction than a benefit. I would like to revisit this in the future though, because
//! this is interesting.


struct TransformClosureHolder<T> {
	transform: T
}

#[async_trait]
impl<T> Transform for TransformClosureHolder<T>
where
	T: TransformClosure + Send + Sync,
{
	async fn base_transform<B>(&mut self, r: &mut dyn HalfStreamRead, w: &mut [u8]) -> Result<usize> {
		(self.transform)(r, w)
	}
}

/// Create a stream transform from a closure that will be called continuously to transform data.
/// this is a uni-directional transform, so you will need to call it twice to create a
/// bi-directional transform. Also, the transform closure is expected to operate by performing a
/// SINGLE read (filling a buffer if necessary) and writing the transformed data to the output
/// buffer. then returning the number of bytes written.
pub fn from_transform_closure<F>(transform: F) -> impl HalfPtStream
where
    F: TransformClosure + Send + Sync + 'static,
{
	let t = Box::new(TransformClosureHolder{transform: Box::new(transform)});
	from_transform(t)
}

pub trait TransformClosure = FnMut(&mut dyn HalfStreamRead, &mut [u8]) -> Box<dyn std::future::Future<Output = Result<usize>>>;

impl AsPtStream for (Box<dyn TransformClosure>, Box<dyn TransformClosure>) {
	fn as_pt_stream(self) -> Result<impl PtStream>{
		let (a, b) = self;
		let a_to_b = from_transform_closure(a);
		let b_to_a = from_transform_closure(b);
		Ok(DuplexBuilder::from_halves(a_to_b, b_to_a))
	}
}
