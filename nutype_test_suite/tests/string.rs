use nutype::nutype;

#[cfg(test)]
mod sanitizers {
    use super::*;

    #[test]
    fn test_trim() {
        #[nutype(sanitize(trim))]
        pub struct Name(String);

        assert_eq!(Name::from("").into_inner(), "");
        assert_eq!(Name::from("\n\t ").into_inner(), "");
        assert_eq!(Name::from(" ! ").into_inner(), "!");
        assert_eq!(Name::from(" foo \n bar ").into_inner(), "foo \n bar");
    }

    #[test]
    fn test_lowercase() {
        #[nutype(sanitize(lowercase))]
        pub struct Name(String);

        assert_eq!(Name::from("  ").into_inner(), "  ");
        assert_eq!(Name::from("Hello THERE").into_inner(), "hello there");
    }

    #[test]
    fn test_uppercase() {
        #[nutype(sanitize(uppercase))]
        pub struct Name(String);

        assert_eq!(Name::from(" ").into_inner(), " ");
        assert_eq!(Name::from("Hello THERE").into_inner(), "HELLO THERE");
    }

    #[cfg(test)]
    mod with {
        use super::*;

        #[test]
        fn test_with_closure_with_explicit_type() {
            #[nutype(sanitize(with = |s: String| s.trim().to_uppercase() ))]
            pub struct Name(String);

            assert_eq!(Name::from(" Anton\n\n").into_inner(), "ANTON");
        }

        #[test]
        fn test_closure_with_no_type() {
            #[nutype(sanitize(with = |s| s.trim().to_uppercase() ))]
            pub struct Name(String);

            assert_eq!(Name::from(" Anton\n\n").into_inner(), "ANTON");
        }

        fn sanitize_name(raw_name: String) -> String {
            raw_name.trim().to_uppercase()
        }

        #[test]
        fn test_with_function() {
            #[nutype(sanitize(with = sanitize_name))]
            pub struct Name(String);

            assert_eq!(Name::from(" Anton\n\n").into_inner(), "ANTON");
        }
    }

    #[test]
    fn test_many_sanitizers() {
        #[nutype(sanitize(trim, uppercase, with = |s| s[1..=2].to_string()))]
        pub struct Country(String);

        assert_eq!(Country::from(" Deutschland ").into_inner(), "EU");
    }
}

#[cfg(test)]
mod validators {
    use super::*;

    #[test]
    fn test_max_len() {
        #[nutype(validate(max_len = 5))]
        pub struct Name(String);

        assert_eq!(Name::try_from("Anton").unwrap().into_inner(), "Anton");
        assert_eq!(Name::try_from("Serhii"), Err(NameError::TooLong));
    }

    #[test]
    fn test_min_len() {
        #[nutype(validate(min_len = 6))]
        pub struct Name(String);

        assert_eq!(Name::try_from("Anton"), Err(NameError::TooShort));
        assert_eq!(Name::try_from("Serhii").unwrap().into_inner(), "Serhii");
    }

    #[test]
    fn test_present() {
        #[nutype(validate(present))]
        pub struct Name(String);

        assert_eq!(Name::try_from(""), Err(NameError::Missing));
        assert_eq!(Name::try_from(" ").unwrap().into_inner(), " ");
        assert_eq!(Name::try_from("Julia").unwrap().into_inner(), "Julia");
    }

    #[test]
    fn test_many_validators() {
        #[nutype(validate(min_len = 3, max_len = 6))]
        pub struct Name(String);

        assert_eq!(Name::try_from("Jo"), Err(NameError::TooShort));
        assert_eq!(Name::try_from("Friedrich"), Err(NameError::TooLong));
        assert_eq!(Name::try_from("Julia").unwrap().into_inner(), "Julia");
    }
}

#[cfg(test)]
mod complex {
    use super::*;

    #[test]
    fn test_sanitizers_and_validators() {
        #[nutype(
            sanitize(trim, with = |s| s.to_uppercase())
            validate(present, max_len = 6)
        )]
        pub struct Name(String);

        assert_eq!(Name::try_from("    "), Err(NameError::Missing));
        assert_eq!(Name::try_from("Willy Brandt"), Err(NameError::TooLong));
        assert_eq!(Name::try_from("   Brandt  ").unwrap().into_inner(), "BRANDT");
    }
}
