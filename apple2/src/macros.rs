#[macro_export]
macro_rules! send_message {
    ($sender: expr, $message: expr)=> {
        if let Some(sender) = $sender {
            sender.send($message).unwrap();
        }
    }
}