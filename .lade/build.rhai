system("cargo", ["build", "-r"])

let move_from = path("./target/release/ep");
let move_to = path(".");

move_file(move_from.to_string(), move_to.to_string());
