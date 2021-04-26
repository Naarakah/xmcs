fn main() {
    use xmcs::dag::xmcsk;

    let s1 = "BDABCBADBCBADBADBDACDABCDABDCBACDBACDBCCDABCCCBADBCABDCABC"
        .chars()
        .collect::<Vec<_>>();
    let s2 = "BDABCBACCBCBADBABDBDDACCBCDADCBAACDBADCDBCCDABCCBADBCCCADCDABC"
        .chars()
        .collect::<Vec<_>>();
    let s3 = "BDABCBAABCBADADBADBDACBACDADCCBACDBACDBCBACDABCCABADBCADBACABC"
        .chars()
        .collect::<Vec<_>>();
    let s4 = "BDABBACBABCCBADBABBDBDACBCDADCBDACDBACDBBCACDABACCBADCBCADCDBABC"
        .chars()
        .collect::<Vec<_>>();

    //let s1 = "CABDCDA".chars().collect::<Vec<_>>();
    //let s2 = "ABCBDD".chars().collect::<Vec<_>>();
    //let res = xmcs::dag::xmcs2(45, &s1, &s2);

    let res = xmcsk(45, &[&s1, &s2, &s3, &s4]);
    res.format_graph(&mut std::io::stdout()).unwrap();

    /* let res = xmcs::set::xmcsk(45, &[&s1, &s2, &s3, &s4]);
    println!("{}", res.len()); */
}
