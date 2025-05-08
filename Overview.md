
Okay, let's break down the control flow, data structures, and abstractions in your Tidal OSC visualizer program.

**Overall Goal:**

The program listens for OSC (Open Sound Control) messages, typically sent by TidalCycles, on a specified IP address and port. It then parses these messages, maintains a state representing the parameters of active sound sources, and displays this information in a formatted way in the terminal.

**File Breakdown and Control Flow:**

1.  **`main.rs` (Entry Point & Main Loop)**
    *   **Initialization:**
        *   Parses command-line arguments to get the IP address and port to listen on (e.g., `127.0.0.1:57120`).
        *   Binds a `UdpSocket` to this address.
        *   Initializes core data structures:
            *   `msg_window: DirtWindow` (from `params.rs`): A `VecDeque` (double-ended queue) to store a sliding window of the most recently received `DirtMessage`s. `WINDOW_SIZE` (currently 100) defines its capacity.
            *   `time_window: VecDeque<u128>`: A `VecDeque` to store the nanosecond timestamps between recent incoming messages, used to calculate an average messages per second rate. `TIME_WINDOW_SIZE` (currently 10) defines its capacity.
            *   `dirt_state: DirtState` (from `params.rs`): A `HashMap` that stores the current state of all known sound sources. The key is a `String` (the `_id_` of the sound source), and the value is a `DirtMessage`.
    *   **Main Loop (`loop`)**:
        1.  **Receive OSC Packet:** `sock.recv_from(&mut buf)` blocks until a UDP packet is received.
        2.  **Terminal Sizing:** Gets current terminal dimensions using `crossterm::terminal::size`.
        3.  **Display (Current):** Calls `dirt_display::display_text(&format!("{:?}",dirt_state));` to print a debug representation of `dirt_state`. This is followed by a short `thread::sleep`.
        4.  **Metrics Update:**
            *   Updates `bytes_recieved_in_sec`.
            *   Prints the average messages per second, calculated from `avg_elapsed`.
        5.  **Decode OSC Packet:** `rosc::decoder::decode_udp(&buf[..size]).unwrap()` decodes the raw bytes into an `OscPacket`.
        6.  **Handle Packet (`handle_packet` function):**
            *   This function is called with the decoded `OscPacket`, a mutable reference to `dirt_state`, and a mutable reference to `msg_window`.
            *   **If `OscPacket::Message(msg)`:**
                *   `params::update_dirt_state(dirt_state, msg.args, msg_window)`: This is a crucial function that updates the application's state based on the incoming message. (More details in the `params.rs` section).
                *   `dirt_display::display_dirt(dirt_state, msg_window)`: This function renders the current `dirt_state` and `msg_window` to the terminal. (More details in the `dirt_display.rs` section).
            *   **If `OscPacket::Bundle(_bundle)`:** Currently, bundles are received but not processed further (commented-out print statement).
        7.  **Time Tracking:**
            *   Calculates `last_elapsed` (time since the last message).
            *   Updates `avg_elapsed` by taking the average of the times stored in `time_window`.
            *   Pushes `last_elapsed` to the front of `time_window` and removes the oldest element if the window is full.
        8.  **Loop Continuation:** The loop continues to wait for the next packet.
    *   **Error Handling:** If `sock.recv_from()` returns an error, the loop breaks, and the program likely exits.

2.  **`params.rs` (Data Structures & State Management Logic)**
    *   **Core Data Structures:**
        *   `DirtValue`: An enum representing the possible types of values for a Tidal parameter: `DI(i32)` (integer), `DF(f32)` (float), or `DS(String)` (string).
        *   `DirtParamName`: A type alias for `String`, representing the name of an OSC parameter (e.g., "s", "gain", "pan").
        *   `DirtMessage`: A type alias for `HashMap<DirtParamName, DirtValue>`. This is a central data structure representing all parameters and their values for a *single sound source* at a specific moment (or as received in one OSC message). The key `_id_` within this map usually identifies the specific sound source or event.
        *   `DirtState`: A type alias for `HashMap<String, DirtMessage>`. This represents the *overall current state* of all sound sources being tracked. The key is the unique identifier of the sound source (extracted as `_id_`), and the value is its latest `DirtMessage`.
        *   `DirtWindow`: A type alias for `VecDeque<DirtMessage>`. This acts as a circular buffer holding the most recent `N` `DirtMessage`s received, irrespective of their `_id_`.
    *   **Trait:**
        *   `GetDirtValue`: A trait implemented for `&DirtMessage`. It provides methods (`display_i32`, `display_f32`, `display_string`, `display_raw`) to extract and format specific parameter values from a `DirtMessage`. This allows the display logic in `dirt_display.rs` to request data without needing to know the internal `DirtValue` enum variants.
    *   **Key Functions:**
        *   `to_dirt_value(&OscType) -> DirtValue`: Converts an `rosc::OscType` into the internal `DirtValue` enum.
        *   `get_param_name(&OscType) -> DirtParamName`: Extracts the parameter name (which is expected to be an `OscType::String`).
        *   `to_dirt_message(Vec<OscType>) -> DirtMessage`: Converts a vector of `OscType` arguments (from an OSC message) into a `DirtMessage` map. It expects arguments in pairs: param name (String), param value.
        *   `update_dirt_message(&mut DirtMessage, Vec<OscType>)`: Clears an existing `DirtMessage` and repopulates it with new parameters from an OSC message argument vector.
        *   `get_id(OscType, OscType) -> String`: Extracts the sound source identifier. It expects the first OSC argument to be the string `"_id_"` and the second to be its string value. Returns an empty string if the first parameter isn't `"_id_"`.
        *   `update_dirt_state(&mut DirtState, Vec<OscType>, &mut DirtWindow)`:
            1.  Extracts the `id` using `get_id()`. If `id` is empty, it returns early (ignores messages not conforming to the `_id_` convention).
            2.  **Window Management:** If `msg_window` exceeds a certain size (currently hardcoded to 10, not `WINDOW_SIZE`), it removes the oldest message (`pop_back`). It then attempts to get the `_id_` from this removed message and inserts an *empty* `HashMap` into `dirt_state` for that `id`. This effectively clears the state of very old messages that have fallen out of the window.
            3.  **State Update:**
                *   If the `id` already exists as a key in `dirt_state`:
                    *   It gets a mutable reference to the existing `DirtMessage`.
                    *   Calls `update_dirt_message()` to update this `DirtMessage` with the new parameters.
                    *   Pushes a *clone* of the updated `DirtMessage` to the front of `msg_window`.
                *   If the `id` is new:
                    *   It creates a new `DirtMessage` using `to_dirt_message()`.
                    *   Inserts this new `DirtMessage` (cloned) into `dirt_state` with its `id`.
                    *   Pushes the new `DirtMessage` (cloned) to the front of `msg_window`.
        *   `new_dirt_window(size: usize) -> DirtWindow`: A simple constructor for `DirtWindow`.

3.  **`dirt_display.rs` (Terminal Rendering Logic)**
    *   **Core Functions:**
        *   `display_text(msg: &String)`: A utility function that clears the terminal from the cursor down, hides the cursor, writes the message, moves the cursor to (0,0), and shows the cursor. It uses `crossterm` for terminal manipulation.
        *   `display_dirt(dirt_state: &DirtState, dirt_window: &DirtWindow)`:
            1.  Initializes an empty `full_str`.
            2.  If `dirt_window` is not empty, it takes the front (most recent) message.
            3.  Displays global information like the "cycle" parameter using `msg.display_f32("cycle", display_cycle)`.
            4.  Iterates through the `dirt_state` `HashMap`. For each `(_id, msg)` pair:
                *   Calls `display_dirt_message(msg)` to get a formatted string for that sound source's state.
                *   Appends this to `full_str`.
            5.  Appends a raw display of the most recent message from `dirt_window` using `msg.display_raw()`.
            6.  Calls `display_text(&full_str)` to render everything.
        *   `display_dirt_message(msg: &DirtMessage) -> String`:
            1.  This is the workhorse for formatting a single `DirtMessage`.
            2.  It checks the `_id_` and skips display if it's "tick".
            3.  It dynamically calculates `cols` (available width for display, leaving `RIGHT_SPACE` for parameter names).
            4.  It then uses the `GetDirtValue` trait methods (e.g., `msg.display_string("_id_", ...)`, `msg.display_f32("delta", ...)`) to extract specific parameters.
            5.  For each parameter, it passes a lambda function that defines *how* that parameter should be formatted (e.g., using `display_bar_float`, `display_bin_float`, or simple string formatting).
            6.  Builds up a string containing all the formatted parameters for this `DirtMessage`.
    *   **Formatting Helpers:**
        *   `display_bar_float(f: &f32, min: f32, max: f32) -> String`: Creates a text-based horizontal bar graph for a float value, scaling it to the available terminal width (`cols`). Uses characters from `string_constants::BAR_CHARS`.
        *   `display_bar_int(i: &i32, min: i32, max: i32) -> String`: Creates a text representation showing an integer value within a range, highlighting the current value.
        *   `display_bin_float(f: &f32) -> String`: Formats the integer part of a float as a 16-bit binary string.
        *   `display_cycle(f: &f32) -> String`: Special formatting for the "cycle" parameter, showing a bar for the fractional part and various modulo values.
        *   `display_cycle_bin(f: &f32) -> String`: Alternative "cycle" display using binary.
        *   `get_box_string(val: usize) -> String`: Helper to generate the bar graph string using `BOX` and `BAR_CHARS`.
        *   `remap_range(...)`: Utility to remap a value from one numerical range to another.
        *   `float_mod(...)`: Custom modulo for floats.

4.  **`string_constants.rs` (UI Character Constants)**
    *   `BOX: &'static str = "█"`: Full block character.
    *   `BAR_CHARS: &'static [&'static str]`: An array of fractional block characters ("▏" to "█") used for finer granularity in text-based bar graphs.

**Key Abstractions:**

*   **State Representation (`DirtState`, `DirtMessage`, `DirtValue`)**: These structures abstract the raw OSC messages into a more organized and typed representation of the sound synthesis parameters. `DirtState` provides a snapshot of all active sounds.
*   **Sliding Window (`DirtWindow`)**: This abstracts the concept of "recent messages," useful for displaying the latest activity and for a simple form of state cleanup (removing very old entries from `DirtState`).
*   **Display Decoupling (`GetDirtValue` trait)**: The `GetDirtValue` trait allows `DirtMessage` to provide its data for display without being tightly coupled to the specific formatting logic. The display functions in `dirt_display.rs` decide *how* to render the data obtained via this trait.
*   **Visual Components (Bar Graphs, etc.)**: Functions like `display_bar_float` abstract the logic for creating specific visual representations of data within the terminal.
*   **Terminal Abstraction (`crossterm`)**: The `crossterm` library is used to abstract away platform-specific terminal control codes, making operations like clearing the screen and moving the cursor portable. This is mostly encapsulated within `display_text` and used directly in `display_dirt_message` for calculating available width.

**Control Flow Summary:**

1.  **Setup:** `main.rs` sets up the UDP listener and initial data structures.
2.  **Listen & Receive:** The main loop in `main.rs` waits for OSC packets.
3.  **Decode:** Raw OSC data is decoded.
4.  **State Update:** `params::update_dirt_state` is called for each message.
    *   It extracts an `_id_`.
    *   It updates the `msg_window` (circular buffer of recent messages).
    *   It updates or creates an entry in `dirt_state` for the given `_id_` using the new parameter values.
5.  **Render:** `dirt_display::display_dirt` is called.
    *   It iterates through `dirt_state` (and uses `msg_window` for some global info).
    *   For each sound source's `DirtMessage`, `dirt_display::display_dirt_message` formats its parameters into a string, using helper functions for bar graphs, binary representations, etc.
    *   The combined string is printed to the terminal using `dirt_display::display_text`.
6.  **Repeat:** The loop continues.

This system is designed to react to incoming OSC messages, maintain a representation of the current sound parameters, and continuously refresh the terminal display with this information.
