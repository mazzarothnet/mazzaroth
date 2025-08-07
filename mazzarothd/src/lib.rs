pub mod api;
pub mod gossip;
pub mod state;

const MAZZAROTH_HTTP_PORT: &str = "MAZZAROTH_HTTP_PORT";
const MAZZAROTH_HTTP_PORT_DEFAULT: &str = "8080";

const MAZZAROTH_UDP_PORT: &str = "MAZZAROTH_UDP_PORT";
const MAZZAROTH_UDP_PORT_DEFAULT: &str = "8081";

const SEED_NODE_ADDR: &str = "SEED_NODE_ADDR";
const SEED_NODE_ADDR_DEFAULT: &str = "127.0.0.1:8081";
