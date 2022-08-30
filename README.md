`un_ros_poc`
====================

Proof of concept of ROS2 usage via the Zenoh-dds bridge in Rust. To try:

1. Run zenohd (branch = "api-changes")
2. zenoh-bridge-dds (branch = "api-changes")
3. Run some ros publishers
   - `ros2 run demo_nodes_cpp talker`
   - `ros2 topic pub /number std_msgs/msg/UInt16 "{ 'data': 99 }"`
   - `ros2 topic pub /pose geometry_msgs/msg/Pose "{ 'position': { 'x': 1.0, 'y': 2.0, 'z': 3.0 }, orientation: { 'x': 0.1, 'y': 0.2, 'z': 0.3, 'w': 0.4 } }"`
4. Edit zenoh connection settings in `main.rs` if necessary.
5. cargo run
6. Try to change the message structs in `main.rs`
