//! Implements a wrapper struct container for internal getter.

#[cfg(test)]
mod test {
    use { Getter, Factory, AsFactoryExt };
    use std::any::Any;

    #[test]
    fn should_get_correct_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.take(), "HAI");
    }

    #[test]
    fn cloned_factory_should_get_the_same_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.clone().take(), factory.take());
    }

    #[test]
    fn should_be_able_to_downcast_from_any() {
        let boxany = box create_with_val("HAI") as Box<Any>;
        let downcasted = boxany.as_factory_of::<String>().unwrap();

        assert_eq!(downcasted.take(), "HAI");
    }

    fn create_with_val(val: &str) -> Factory<String> {
        Factory::new(box ValContainer { val: val.to_string() })
    }

    struct ValContainer {
        val: String,
    }

    impl Getter<String> for ValContainer {
        fn take(&self) -> String {
            self.val.clone()
        }

        fn boxed_clone<'r>(&self) -> Box<Getter<String> + 'r> {
            box ValContainer {
                val: self.val.clone(),
            }
        }
    }
}
