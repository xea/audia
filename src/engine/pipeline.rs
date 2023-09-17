pub trait Node {

}

pub trait Something<T> {

    type E;

    fn do_something(item: Self::E) -> T;
}

#[cfg(test)]
mod tests {

    #[test]
    fn ping() {

    }
}