// e2e_143 - std net/netip: address, prefix and addr:port codecs
module e2e_143;

import std::net::netip;

fn text(s: String) -> String {
    let ip = netip::ParseIP(s);
    if ip.ok == false {
        return "!";
    }
    return netip::ToString(ip);
}

pub fn main() -> void {
    assert(text("1.2.3.4") == "1.2.3.4", "v4 identity");
    assert(text("001.2.3.4") == "!", "v4 leading zero");
    assert(text("256.1.1.1") == "!", "v4 range");
    assert(text("1.2.3") == "!", "v4 arity");
    assert(text("") == "!", "empty");
    assert(text("::") == "::", "unspecified");
    assert(text("::1") == "::1", "loopback v6");
    assert(text("2001:DB8::1") == "2001:db8::1", "uppercase + compress");
    assert(
        text("2001:0db8:0000:0000:0000:ff00:0042:8329") == "2001:db8::ff00:42:8329",
        "rfc 5952 longest run",
    );
    assert(text("1:0:0:2:0:0:0:3") == "1:0:0:2::3", "tie goes to first run");
    assert(text("1:2:3:4:5:6:7:8") == "1:2:3:4:5:6:7:8", "no run");
    assert(text("::ffff:1.2.3.4") == "1.2.3.4", "v4-mapped unmaps");
    assert(text("1:2:3:4:5:6:1.2.3.4") == "1:2:3:4:5:6:102:304", "embedded v4");
    assert(text("2001:db8:::1") == "!", "double compression");
    assert(text("12345678::") == "!", "five hex digits");
    assert(netip::IsIP("fe80::1"), "IsIP");
    assert(netip::IsIP("fe80::1%eth0") == false, "zone rejected");

    let v4 = netip::ParseIP("203.0.113.7");
    assert(netip::Is4(v4) && netip::Bits(v4) == 32, "is4/bits");
    assert(netip::IsLoopback(netip::ParseIP("127.9.9.9")), "v4 loopback");
    assert(netip::IsMulticast(netip::ParseIP("ff02::1")), "v6 multicast");
    assert(netip::IsUnspecified(netip::ParseIP("0.0.0.0")), "v4 unspecified");
    assert(netip::IsLinkLocalUnicast(v4) == false, "v4 not link-local");
    assert(netip::IsLinkLocalUnicast(netip::ParseIP("fe80::2")), "v6 link-local");
    let a16 = netip::As16(v4);
    assert(a16.len() == 16 && a16[10] == 255 && a16[15] == 7, "as16");

    let p = netip::ParsePrefix("10.0.0.0/8");
    assert(p.ok && netip::PrefixString(p) == "10.0.0.0/8", "prefix v4");
    assert(netip::Contains(p, netip::ParseIP("10.1.2.3")), "contains in");
    assert(netip::Contains(p, netip::ParseIP("11.0.0.1")) == false, "contains out");
    let m = netip::Masked(netip::ParsePrefix("10.1.2.3/8"));
    assert(netip::PrefixString(m) == "10.0.0.0/8", "masked v4");
    let m24 = netip::Masked(netip::ParsePrefix("192.168.1.199/24"));
    assert(netip::PrefixString(m24) == "192.168.1.0/24", "masked /24");
    let p6 = netip::ParsePrefix("2001:db8::/32");
    assert(p6.ok && netip::Contains(p6, netip::ParseIP("2001:0db8:abcd::5")), "contains v6");
    assert(netip::Contains(p6, netip::ParseIP("2002:db8::5")) == false, "contains v6 out");
    let p6m = netip::Masked(netip::ParsePrefix("2001:db8:1234::/48"));
    assert(netip::PrefixString(p6m) == "2001:db8:1234::/48", "masked v6");
    let pBad = netip::ParsePrefix("1.2.3.4/33");
    assert(pBad.ok == false, "prefix range");

    let ap = netip::ParseAddrPort("1.2.3.4:80");
    assert(ap.ok && ap.port == 80 && netip::AddrPortString(ap) == "1.2.3.4:80", "addrport v4");
    let ap6 = netip::ParseAddrPort("[::1]:443");
    assert(ap6.ok && netip::AddrPortString(ap6) == "[::1]:443", "addrport v6");
    assert(netip::ParseAddrPort("1.2.3.4:70000").ok == false, "port range");
    assert(netip::ParseAddrPort("[::1]80").ok == false, "missing colon");
    assert(netip::ParseAddrPort("::1").ok == false, "no port");

    let built = netip::FromBytes([192, 168, 0, 1]);
    assert(netip::ToString(built) == "192.168.0.1", "FromBytes v4");
    assert(netip::FromBytes([1, 2, 3]).ok == false, "FromBytes arity");
}
