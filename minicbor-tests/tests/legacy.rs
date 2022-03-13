use minicbor::decode::{Token, Tokenizer};
use minicbor::legacy;
use quickcheck::quickcheck;

fn assert_ip_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(2)))));
    match it.next() {
        Some(Ok(Token::U8(0))) => assert_ipv4_layout(it),
        Some(Ok(Token::U8(1))) => assert_ipv6_layout(it),
        other                  => panic!("unexpected ip address variant: {other:?}")
    }
}

fn assert_ipv4_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(4)))));
    for _ in 0 .. 4 {
        assert!(matches!(it.next(), Some(Ok(Token::U8(_)))))
    }
}

fn assert_ipv6_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(16)))));
    for _ in 0 .. 16 {
        assert!(matches!(it.next(), Some(Ok(Token::U8(_)))))
    }
}

fn assert_sockaddr_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(2)))));
    match it.next() {
        Some(Ok(Token::U8(0))) => assert_sockaddrv4_layout(it),
        Some(Ok(Token::U8(1))) => assert_sockaddrv6_layout(it),
        other                  => panic!("unexpected socket address variant: {other:?}")
    }
}

fn assert_sockaddrv4_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(2)))));
    assert_ipv4_layout(it);
    assert!(matches!(it.next(), Some(Ok(Token::U8(_) | Token::U16(_)))))
}

fn assert_sockaddrv6_layout(it: &mut Tokenizer) {
    assert!(matches!(it.next(), Some(Ok(Token::Array(2)))));
    assert_ipv6_layout(it);
    assert!(matches!(it.next(), Some(Ok(Token::U8(_) | Token::U16(_)))))
}

#[test]
fn ip_legacy_layout() {
    fn property(a: std::net::IpAddr) {
        let v = minicbor::to_vec(&legacy::IpAddr(a)).unwrap();
        assert_ip_layout(&mut Tokenizer::new(&v));
    }
    quickcheck(property as fn(std::net::IpAddr))
}

#[test]
fn ipv4_legacy_layout() {
    fn property(a: std::net::Ipv4Addr) {
        let v = minicbor::to_vec(&legacy::Ipv4Addr(a)).unwrap();
        assert_ipv4_layout(&mut Tokenizer::new(&v))
    }
    quickcheck(property as fn(std::net::Ipv4Addr))
}

#[test]
fn ipv6_legacy_layout() {
    fn property(a: std::net::Ipv6Addr) {
        let v = minicbor::to_vec(&legacy::Ipv6Addr(a)).unwrap();
        assert_ipv6_layout(&mut Tokenizer::new(&v))
    }
    quickcheck(property as fn(std::net::Ipv6Addr))
}

#[test]
fn sockaddr_legacy_layout() {
    fn property(a: std::net::SocketAddr) {
        let v = minicbor::to_vec(&legacy::SocketAddr(a)).unwrap();
        assert_sockaddr_layout(&mut Tokenizer::new(&v))
    }
    quickcheck(property as fn(std::net::SocketAddr))
}

#[test]
fn sockaddrv4_legacy_layout() {
    fn property(a: std::net::SocketAddrV4) {
        let v = minicbor::to_vec(&legacy::SocketAddrV4(a)).unwrap();
        assert_sockaddrv4_layout(&mut Tokenizer::new(&v))
    }
    quickcheck(property as fn(std::net::SocketAddrV4))
}

#[test]
fn sockaddrv6_legacy_layout() {
    fn property(a: std::net::SocketAddrV6) {
        let v = minicbor::to_vec(&legacy::SocketAddrV6(a)).unwrap();
        assert_sockaddrv6_layout(&mut Tokenizer::new(&v))
    }
    quickcheck(property as fn(std::net::SocketAddrV6))
}

