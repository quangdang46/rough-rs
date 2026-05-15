pub fn random_seed() -> u64 {
    #[cfg(feature = "rand")]
    {
        use rand::Rng;
        rand::thread_rng().gen_range(0..(1_u64 << 31))
    }

    #[cfg(not(feature = "rand"))]
    {
        0
    }
}
