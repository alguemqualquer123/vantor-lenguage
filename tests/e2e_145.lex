// e2e_145 - std sync/pool + std encoding/binary
module e2e_145;

import std::sync::pool;
import std::encoding::binary;

pub fn main() -> void {
    // Value-semantic pool: every op returns the new pool, reassign it.
    let p = pool::New();
    assert(pool::Len(p) == 0, "empty pool");
    let miss = pool::Get(p);
    assert(miss.ok == false && pool::Len(miss.pool) == 0, "get on empty");
    p = pool::Put(p, "one");
    p = pool::Put(p, "two");
    p = pool::Put(p, "three");
    assert(pool::Len(p) == 3, "three pooled");
    let g1 = pool::Get(p);
    assert(g1.ok && g1.value == "three" && pool::Len(g1.pool) == 2, "lifo first");
    let g2 = pool::Get(g1.pool);
    assert(g2.value == "two", "lifo second");
    let g3 = pool::Get(g2.pool);
    assert(g3.value == "one" && pool::Len(g3.pool) == 0, "lifo drains");

    // Fixed-size integer codecs (Go's binary.BigEndian / LittleEndian).
    let b32 = binary::PutBE32(305419896);
    assert(b32[0] == 18 && b32[1] == 52 && b32[2] == 86 && b32[3] == 120, "be32 bytes");
    assert(binary::Be32(b32) == 305419896, "be32 round trip");
    let l32 = binary::PutLE32(305419896);
    assert(l32[0] == 120 && l32[3] == 18, "le32 order");
    assert(binary::Le32(l32) == 305419896, "le32 round trip");
    let b64 = binary::PutBE64(1311768467463790320);
    assert(b64.len() == 8 && b64[0] == 18 && b64[7] == 240, "be64 bytes");
    assert(binary::Be64(b64) == 1311768467463790320, "be64 round trip");
    let l64 = binary::PutLE64(1311768467463790320);
    assert(l64[0] == 240 && l64[7] == 18, "le64 order");
    assert(binary::Le64(l64) == 1311768467463790320, "le64 round trip");
    assert(binary::Be16(binary::PutBE16(4660)) == 4660, "be16");
    assert(binary::Le16(binary::PutLE16(4660)) == 4660, "le16");
    assert(binary::Be16(binary::PutBE16(0)) == 0, "zero");
    let all = binary::WriteBE32All([1, 256, 305419896]);
    assert(all.len() == 12 && all[0] == 0 && all[3] == 1 && all[6] == 1 && all[11] == 120, "write all");
    assert(binary::Be32([0, 0, 1, 0]) == 256, "read slice");
}
