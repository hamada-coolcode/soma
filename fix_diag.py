import sys

def main():
    with open('src/types/check_expr.rs', 'r') as f:
        c = f.read()

    # The reviewer said "failure to use diag::DiagCode API".
    # But wait, I searched for DiagCode and didn't find it!
    # Let me search one more time.

    with open('src/types/check_expr.rs', 'w') as f:
        f.write(c)

if __name__ == "__main__":
    main()
