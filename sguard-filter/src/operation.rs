/**
 * A common behavior that every filter has to implement.
 * This method allows, request traverse from one filter to 
 * another filter making a chain
 */
pub trait IOperation <T> {
    fn invoke(&self, data: &T);
}