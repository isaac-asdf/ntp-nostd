#![no_std]

#[derive(PartialEq, Debug)]
pub struct PacketHeaders {
    /// indicate how leap second will be displayed
    li: LI, // 2 bit
    /// stratum version number
    vn: u8, // 3 bit
    /// set at 3 for client
    mode: Mode, // 3 bit
    /// Sounds like it has some goofy mapping [see figure 11](https://www.rfc-editor.org/rfc/rfc5905#section-7.1)
    stratum: Stratum, // 8 bit
    /// max interval between successive messages, in log2 seconds.
    /// suggested defaults are [6, 10] for [min, max]
    poll: i8, // 8 bit
    /// epresenting the precision of the system clock, in log2 seconds.
    precision: i8, // 8 bit
    /// round trip delay to reference clock, in NTP short format
    root_delay: u32,
    /// total dispersion to reference clock, in NTP short format
    root_dispersion: u32,
    /// interpretation depends on value in stratum field
    ref_id: u32, // 32 bit
    /// time when the system clock was last set or corrected, in NTP timestamp format
    ref_time: u64,
    /// Time at the client when the request departed for the server, in NTP timestamp format
    origin_time: u64,
    /// Time at the server when the request arrived from the client, in NTP timestamp format.
    rx_time: u64,
    /// Time at the server when the response left for the client, in NTP timestamp format.
    pub tx_time_seconds: u32,
    tx_time_fraction: u32,
    /// Time at the client when the reply arrived from the server, in NTP timestamp format.
    /// NOT included in packet header, client to set upon packet arrival
    dst_time: u64,
    /// part of msg digst?
    key_id: u32,
    /// md5 hash of message?
    msg_dgst: u128,
}

impl PacketHeaders {
    /// get the unix timestamp based on the time the response left the server
    pub fn get_unix_timestamp(self) -> u32 {
        self.tx_time_seconds - UNIX_OFFSET
    }
}

#[derive(PartialEq, Debug)]
pub struct ExtensionField<'a> {
    data_type: u16,
    data_length: u16,
    data: &'a u8,
}

#[derive(PartialEq, Debug)]
pub struct NtpServerResponse<'a> {
    pub headers: PacketHeaders,
    pub extension_fields: Option<[ExtensionField<'a>; 2]>,
}

impl From<&[u8]> for NtpServerResponse<'_> {
    fn from(value: &[u8]) -> Self {
        let mut iter = value.iter();
        let li_vn_mode = iter.next().unwrap();

        // Extract the first two bits into the LI
        let li = (li_vn_mode >> 6) & 0b11;
        let li = li as u8;
        let li: LI = li.into();

        // Extract the next three bits for NTP version
        let version = (li_vn_mode >> 3) & 0b111;
        let version = version as u8;

        // Extract the next three bits for the Mode
        let mode = (li_vn_mode) & 0b111;
        let mode = mode as u8;
        let mode: Mode = mode.into();

        let stratum = *iter.next().unwrap();
        let stratum: Stratum = stratum.into();
        // println!("{:?}, {}, {:?}, {:?}", li, version, mode, stratum);

        let poll = *iter.next().unwrap() as i8;
        let precision = *iter.next().unwrap() as i8;

        let root_delay = combine_u8s(&mut iter);
        let root_dispersion = combine_u8s(&mut iter);
        let ref_id = combine_u8s(&mut iter);

        // get times
        let ref_seconds_1 = combine_u8s(&mut iter);
        let ref_seconds_2 = combine_u8s(&mut iter);
        let ref_time = (u64::from(ref_seconds_1) << 32) | (u64::from(ref_seconds_2));

        let ref_seconds_1 = combine_u8s(&mut iter);
        let ref_seconds_2 = combine_u8s(&mut iter);
        let origin_time = (u64::from(ref_seconds_1) << 32) | (u64::from(ref_seconds_2));

        let ref_seconds_1 = combine_u8s(&mut iter);
        let ref_seconds_2 = combine_u8s(&mut iter);
        let rx_time = (u64::from(ref_seconds_1) << 32) | (u64::from(ref_seconds_2));
        // println!("rx time: {rx_time}");

        let tx_time_seconds = combine_u8s(&mut iter);
        let tx_time_fraction = combine_u8s(&mut iter);

        let headers: PacketHeaders = PacketHeaders {
            li,
            vn: version,
            mode,
            stratum,
            poll,
            precision,
            root_delay,
            root_dispersion,
            ref_id,
            ref_time,
            origin_time,
            rx_time,
            tx_time_seconds,
            tx_time_fraction,
            dst_time: 0,
            key_id: 0,
            msg_dgst: 0,
        };
        NtpServerResponse {
            headers,
            extension_fields: None,
        }
    }
}

fn combine_u8s(iter: &mut core::slice::Iter<'_, u8>) -> u32 {
    let u8_1: u8 = *iter.next().unwrap();
    let u8_2: u8 = *iter.next().unwrap();
    let u8_3: u8 = *iter.next().unwrap();
    let u8_4: u8 = *iter.next().unwrap();
    (u32::from(u8_1) << 24) | (u32::from(u8_2) << 16) | (u32::from(u8_3) << 8) | u32::from(u8_4)
}

#[derive(PartialEq, Debug)]
pub enum LI {
    NoLeap = 0,
    LastMinute61 = 1,
    LastMinute59 = 2,
    UnknownUnsync = 3,
}

impl From<u8> for LI {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoLeap,
            1 => Self::LastMinute61,
            2 => Self::LastMinute59,
            3 => Self::UnknownUnsync,
            _ => panic!("impossible to be here"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Mode {
    Reserved = 0,
    SymActive = 1,
    SymPassive = 2,
    Client = 3,
    Server = 4,
    Broadcast = 5,
    NtpControl = 6,
    ReservedPrivateUse = 7,
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Reserved,
            1 => Self::SymActive,
            2 => Self::SymPassive,
            3 => Self::Client,
            4 => Self::Server,
            5 => Self::Broadcast,
            6 => Self::NtpControl,
            7 => Self::ReservedPrivateUse,
            _ => panic!("impossible"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Stratum {
    UnspecifiedInvalid,
    /// e.g., equipped with a GPS receiver
    PrimaryServer,
    /// via NTP
    SecondaryServer,
    Unsynchronized,
    Reserved,
}

impl From<u8> for Stratum {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::UnspecifiedInvalid,
            1 => Self::PrimaryServer,
            2..=15 => Self::SecondaryServer,
            16 => Self::Unsynchronized,
            _ => Self::Reserved,
        }
    }
}

const UNIX_OFFSET: u32 = 2_208_988_800;
pub const NTP_PORT: u8 = 123;
pub const NTP_VERSION: u8 = 1;
pub const KISS_CODE_DENY: [u8; 4] = *b"DENY";
pub const KISS_CODE_RSTR: [u8; 4] = *b"RSTR";
pub const KISS_CODE_RATE: [u8; 4] = *b"RATE";

pub enum KissCodes {
    UnicastServer,
    AuthFailed,
    AutokeyFailed,
    BroadcastServer,
    CryptoIdAuthFail,
    AccessDenied,
    LostPeerInSymmetricMode,
    AccessRestricted,
    Initializing,
    DynamicallyDiscoveredServer,
    NoKey,
    RateExceeded,
    RemoteAssocAlteration,
    StepTimeChange,
    UnknownKissCode,
}

impl From<&[u8; 4]> for KissCodes {
    fn from(value: &[u8; 4]) -> Self {
        match value {
            b"ACST" => Self::UnicastServer,
            b"AUTH" => Self::AuthFailed,
            b"AUTO" => Self::AutokeyFailed,
            b"BCST" => Self::BroadcastServer,
            b"CRYP" => Self::CryptoIdAuthFail,
            b"DENY" => Self::AccessDenied,
            b"DROP" => Self::LostPeerInSymmetricMode,
            b"RSTR" => Self::AccessRestricted,
            b"INIT" => Self::Initializing,
            b"MCST" => Self::DynamicallyDiscoveredServer,
            b"NKEY" => Self::NoKey,
            b"RATE" => Self::RateExceeded,
            b"RMOT" => Self::RemoteAssocAlteration,
            b"STEP" => Self::StepTimeChange,
            _ => Self::UnknownKissCode,
        }
    }
}

pub fn get_client_request() -> [u8; 48] {
    let mut buff = [0_u8; 48];
    buff[0] = 0b00100011;
    buff[1] = 0;
    buff[2] = 0;
    buff[3] = 0;
    buff[12] = 0;
    buff[13] = 0;
    buff[14] = 0;
    buff[15] = 0;
    buff
}

#[cfg(test)]
mod tests {
    // use tungstenite::{connect, Message};
    // use url::Url;

    use super::*;

    // #[test]
    // fn test_tungstenite() {
    //     let (mut socket, response) =
    //         connect(Url::parse("ws://18.119.130.247:123").unwrap()).expect("Can't connect");

    //     println!("Connected to the server");
    //     println!("Response HTTP code: {}", response.status());
    //     println!("Response contains the following headers:");
    //     for (ref header, _value) in response.headers() {
    //         println!("* {}", header);
    //     }

    //     socket
    //         .send(Message::Text("Hello WebSocket".into()))
    //         .unwrap();
    //     loop {
    //         let msg = socket.read().expect("Error reading message");
    //         println!("Received: {}", msg);
    //     }
    // }

    // #[test]
    // fn test_std_address() {
    //     let port = std::net::UdpSocket::bind("192.168.1.3:34254").unwrap();
    //     port.connect("pool.ntp.org:123").unwrap();
    //     println!("connected to:{:?}", port.peer_addr().unwrap());
    //     let msg = get_client_request();
    //     port.send(&msg).unwrap();
    //     let mut rcvd = [0_u8; 48];
    //     port.recv(&mut rcvd).unwrap();
    //     let _ = NtpServerResponse::from(rcvd.as_ref());
    //     assert!(true);
    // }

    #[test]
    fn test_from_u8() {
        let values: [u8; 48] = [
            36, 3, 0, 232, 0, 0, 5, 139, 0, 0, 0, 39, 10, 72, 8, 222, 232, 139, 229, 188, 150, 26,
            5, 122, 0, 0, 0, 0, 0, 0, 0, 0, 232, 139, 229, 209, 125, 186, 194, 223, 232, 139, 229,
            209, 125, 239, 153, 206,
        ];

        let ntp_response: NtpServerResponse = NtpServerResponse::from(values.as_ref());
        let expected = NtpServerResponse {
            headers: PacketHeaders {
                li: LI::NoLeap,
                vn: 4,
                mode: Mode::Server,
                stratum: Stratum::SecondaryServer,
                poll: 0,
                precision: -24,
                root_delay: 1419,
                root_dispersion: 39,
                ref_id: 172493022,
                ref_time: 16756739436696962426,
                origin_time: 0,
                rx_time: 16756739526482379487,
                tx_time_seconds: 3901482449,
                tx_time_fraction: 2112854478,
                dst_time: 0,
                key_id: 0,
                msg_dgst: 0,
            },
            extension_fields: None,
        };

        assert_eq!(ntp_response, expected);
        assert_eq!(ntp_response.headers.get_unix_timestamp(), 1692493649);
    }

    #[test]
    fn test_tx_time() {
        let values: [u8; 48] = [
            36, 2, 0, 237, 0, 0, 0, 13, 0, 0, 0, 2, 10, 1, 105, 4, 232, 140, 230, 172, 44, 61, 185,
            98, 0, 0, 0, 0, 0, 0, 0, 0, 232, 140, 230, 180, 185, 134, 172, 167, 232, 140, 230, 180,
            185, 136, 186, 218,
        ];
        let ntp_response: NtpServerResponse = NtpServerResponse::from(values.as_ref());

        assert_eq!(ntp_response.headers.tx_time_seconds, 3901548212_u32);
        assert_eq!(ntp_response.headers.get_unix_timestamp(), 1692559412);
    }
}
