mod compiler;
mod definition;
mod env;

fn main() -> Result<(), String> {
    let extension = "scm";
    let path = "../wc-definition";

    let result = lisper::load_directory(extension, path.into())?;
    let result = env::Environment::parse(result)?;
    compiler::compile(result);

    let is_server = false;
    return Ok(());
    loop {
        if is_server {
            // Server logic
            // 1. send the state to the clients
            // 2. send the available actions to the clients
            // 3. send the events to the clients
            // 4. receive the actions from the clients
            // 5. apply the actions to the state
            // 6. repeat
        } else {
            // Client logic
            // 1. get the state from the server
            // 2. get the available actions from the server
            // 3. get the events from the server
            // 4. choose an action
            // 5. send the action to the server
            // 6. repeat
        }
    }

    Ok(())
}

pub struct State;
pub struct Action;
pub struct Event;

pub struct ServerMessage {
    /// The current state of the game
    pub state: State,
    /// The available actions to the player
    pub available_actions: Vec<Action>,
    /// The events that transpired since last state
    pub events: Vec<Event>,
}

pub struct ClientMessage {
    /// The action the player wants to take
    pub action: Action,
}
