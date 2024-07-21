// use super::operation::IOperation;

// pub struct Filter<T> {
//     operations: Vec<Box<dyn IOperation<T>>>,
// }

// impl <T> Filter<T> {
//     pub fn new() -> Self {
//         Self {
//             operations: Vec::new()
//         }
//     }f                       


//     pub fn register(&mut self, operation: Box<dyn IOperation<T>>) {
//         self.operations.push(operation);
//     }
// }

// impl <T> IOperation<T> for Filter<T> {
//     fn invoke(&self, data: &T) {
//         for operation in &self.operations {
//             operation.invoke(data)
//         }
//     }
// }