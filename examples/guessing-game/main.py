
from random import choice

SECRET_NUMBER = choice(range(1, 100))

def main() -> None:
    print("Guess the number!")

    while True:
        guess = int(input("Please input your guess: "))

        print(f"You guessed: {guess}")
        if guess > SECRET_NUMBER: print("Too big!")
        elif guess < SECRET_NUMBER: print("Too small!")
        else:
            print("You win!")
            break

if __name__ == "__main__":
    main()
