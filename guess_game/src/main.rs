use std::cmp::Ordering;
use std::io;
use rand::RngExt;

fn main(){
    println!("Guess the number");

    let mut rng=rand::rng();
    let secret=rng.random_range(1..=10);


    println!("Please input your guess");


    let mut guess=String::new();
    io::stdin().read_line(&mut guess).expect("Failed to read line");

    let guess:u32=guess.trim().parse().expect("type a number");
    println!("You guessed: {}", guess);
    println!("The secret number is: {}", secret);

    match guess.cmp(&secret){
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }

}

