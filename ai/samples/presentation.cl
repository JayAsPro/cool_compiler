class A {
    num : Int <- 10;
    
    number() : Int {
        num
    };
};

class Main inherits IO {
    a : A <- new A;
    
    main() : Object {
        out_int(a.number())
    };
};
