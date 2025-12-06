use super::NetworkError;
use alloc::collections::VecDeque;

/// États TCP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    Closing,
    TimeWait,
    CloseWait,
    LastAck,
}

/// En-tête TCP
#[derive(Debug, Clone)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dest_port: u16,
    pub sequence: u32,
    pub acknowledgment: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

impl TcpHeader {
    pub fn new(src_port: u16, dest_port: u16) -> Self {
        Self {
            src_port,
            dest_port,
            sequence: 0,
            acknowledgment: 0,
            data_offset: 5,
            flags: 0,
            window_size: 65535,
            checksum: 0,
            urgent_pointer: 0,
        }
    }

    pub fn set_syn(&mut self) {
        self.flags |= 0x02;
    }

    pub fn set_ack(&mut self) {
        self.flags |= 0x10;
    }

    pub fn set_fin(&mut self) {
        self.flags |= 0x01;
    }

    pub fn is_syn(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn is_ack(&self) -> bool {
        (self.flags & 0x10) != 0
    }

    pub fn is_fin(&self) -> bool {
        (self.flags & 0x01) != 0
    }
}

/// Socket TCP
pub struct TcpSocket {
    pub state: TcpState,
    pub local_port: u16,
    pub remote_port: u16,
    pub remote_ip: [u8; 4],
    pub send_buffer: VecDeque<u8>,
    pub recv_buffer: VecDeque<u8>,
    pub sequence: u32,
    pub acknowledgment: u32,
}

impl TcpSocket {
    pub fn new() -> Self {
        Self {
            state: TcpState::Closed,
            local_port: 0,
            remote_port: 0,
            remote_ip: [0, 0, 0, 0],
            send_buffer: VecDeque::new(),
            recv_buffer: VecDeque::new(),
            sequence: 0,
            acknowledgment: 0,
        }
    }

    pub fn connect(&mut self, addr: ([u8; 4], u16)) -> Result<(), NetworkError> {
        if self.state != TcpState::Closed {
            return Err(NetworkError::HostUnreachable);
        }

        self.remote_ip = addr.0;
        self.remote_port = addr.1;
        self.state = TcpState::SynSent;

        // TODO: Envoyer le paquet SYN
        Ok(())
    }

    pub fn listen(&mut self, port: u16) -> Result<(), NetworkError> {
        if self.state != TcpState::Closed {
            return Err(NetworkError::PortUnreachable);
        }

        self.local_port = port;
        self.state = TcpState::Listen;
        Ok(())
    }

    pub fn accept(&mut self) -> Result<TcpSocket, NetworkError> {
        if self.state != TcpState::Listen {
            return Err(NetworkError::HostUnreachable);
        }

        // TODO: Accepter une connexion
        Ok(TcpSocket::new())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, NetworkError> {
        if self.state != TcpState::Established {
            return Err(NetworkError::HostUnreachable);
        }

        self.send_buffer.extend(data);
        Ok(data.len())
    }

    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        if self.state != TcpState::Established {
            return Err(NetworkError::HostUnreachable);
        }

        let len = buffer.len().min(self.recv_buffer.len());
        for i in 0..len {
            buffer[i] = self.recv_buffer.pop_front().unwrap();
        }

        Ok(len)
    }

    pub fn close(&mut self) -> Result<(), NetworkError> {
        if self.state == TcpState::Closed {
            return Err(NetworkError::HostUnreachable);
        }

        self.state = TcpState::FinWait1;
        // TODO: Envoyer le paquet FIN
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_tcp_header_creation() {
        let header = TcpHeader::new(1234, 5678);
        assert_eq!(header.src_port, 1234);
        assert_eq!(header.dest_port, 5678);
    }

    #[test_case]
    fn test_tcp_socket_creation() {
        let socket = TcpSocket::new();
        assert_eq!(socket.state, TcpState::Closed);
    }

    #[test_case]
    fn test_tcp_socket_listen() {
        let mut socket = TcpSocket::new();
        assert!(socket.listen(8080).is_ok());
        assert_eq!(socket.state, TcpState::Listen);
    }
}
