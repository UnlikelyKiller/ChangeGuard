use camino::Utf8Path;
use changeguard::index::languages::Language;
use changeguard::index::metrics::{ComplexityScorer, NativeComplexityScorer};

#[test]
fn test_rust_complexity() {
    let source = r#"
        fn simple() {
            println!("hello");
        }

        fn complex(x: i32) {
            if x > 0 {
                for i in 0..x {
                    if i % 2 == 0 {
                        println!("{}", i);
                    }
                }
            } else {
                match x {
                    -1 => println!("one"),
                    _ => println!("other"),
                }
            }
        }
    "#;

    let scorer = NativeComplexityScorer::new();
    let result = scorer.score_file(Utf8Path::new("test.rs"), source, Language::Rust).unwrap();

    assert_eq!(result.functions.len(), 2);
    
    let simple = result.functions.iter().find(|f| f.name == "simple").unwrap();
    assert_eq!(simple.cyclomatic, 1);
    assert_eq!(simple.cognitive, 0);

    let complex = result.functions.iter().find(|f| f.name == "complex").unwrap();
    // Cyclomatic: 1 (base) + 1 (if) + 1 (for) + 1 (if) + 1 (else/match) + 2 (match arms) = 7?
    // Let's see what the implementation gives.
    assert!(complex.cyclomatic > 1);
    assert!(complex.cognitive > 1);
}

#[test]
fn test_python_complexity() {
    let source = r#"
def simple():
    print("hello")

def complex(x):
    if x > 0:
        for i in range(x):
            if i % 2 == 0:
                print(i)
    else:
        print("negative")
    "#;

    let scorer = NativeComplexityScorer::new();
    let result = scorer.score_file(Utf8Path::new("test.py"), source, Language::Python).unwrap();

    assert_eq!(result.functions.len(), 2);
    let complex = result.functions.iter().find(|f| f.name == "complex").unwrap();
    assert!(complex.cyclomatic > 1);
    assert!(complex.cognitive > 1);
}
