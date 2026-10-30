from concurrent.futures import ThreadPoolExecutor


def worker(i: int) -> int:
    total = 0
    for j in range(1_000_000):
        total += j
    print(f"worker {i} sum = {total}")
    return total


def main() -> None:
    # NOTE: the GIL means CPU-bound threads don't truly run in parallel here;
    # use ProcessPoolExecutor for real parallelism. ThreadPool shows the API.
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(worker, range(4)))
    print("total =", sum(results))


if __name__ == "__main__":
    main()
