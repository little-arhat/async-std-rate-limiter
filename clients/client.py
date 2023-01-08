#!/usr/bin/env python3

import os
import sys
import time
import random
import string
import socket
import statistics

from enum import Enum
from argparse import ArgumentParser


SYMBOLS = string.ascii_uppercase
PARSE_SUCCESS = b"0\n"
PARSE_FAILURE = b"1\n"


class Result(Enum):
    SUCCESS = "Success"
    FAILURE = "Failure"


PARSE = {PARSE_SUCCESS: Result.SUCCESS, PARSE_FAILURE: Result.FAILURE}


def stamp_ms(start_s):
    return int((time.time() - start_s) * 1000)


def roundtrip(s, symbol):
    payload = bytes(symbol + "\n", "ascii")
    s.send(payload)
    response = s.recv(64)
    msg = PARSE.get(response)
    if msg is None:
        raise ValueError(
            "Invalid response from server, abort: [{}]".format(str(response, "utf-8"))
        )
    return msg


def session(client, gen):
    try:
        response = None
        epoch_start = time.time()
        while True:
            payload = gen.send(response)
            response = roundtrip(client, payload)
            epoch = stamp_ms(epoch_start)
            print(f"[{epoch}] Sent: {payload}; result: {response}")
    except StopIteration:
        pass
    finally:
        print("Closing connection")
        client.close()


def address():
    addr = os.getenv("SERVER_ADDRESS", "127.0.0.1:31337")
    (host, port) = addr.split(":")
    return (host, int(port))


def mk_client(host, port):
    print(f"Connecting to {host}:{port}")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    print(f"Connected to {host}:{port}")
    return s


def fast_random(delay_ms):
    """
    Sends random symbol with a configured *delay_ms* until terminated
    """
    delay_s = delay_ms / 1000.0
    while True:
        symbol = random.choice(SYMBOLS)
        response = yield symbol
        time.sleep(delay_s)


def cycle(delay_ms):
    """
    Cycles through available symbols with configured *delay_ms* until terminated
    """
    delay_s = delay_ms / 1000.0
    while True:
        for symbol in SYMBOLS:
            response = yield symbol
            time.sleep(delay_s)


def burst(iterations):
    """
    Sends random symbol without delay, trying to compute rate limit.
    Does so *iterations* times with 1s delay between iterations.
    """
    r = []
    for attempt in range(iterations):
        n = 0
        symbol = random.choice(SYMBOLS)
        while True:
            response = yield symbol
            if response is Result.FAILURE:
                r.append(n)
                time.sleep(1)
                break  # out of while loop
            else:
                n += 1
    median = statistics.median(r)
    mean = statistics.mean(r)
    print(f"Computed approx rate limit (p/s): median={median}; mean={mean}")


def symbol(symbol, delay_ms):
    """
    Sends one *symbol* with a *delay_ms*
    """
    delay_s = delay_ms / 1000.0
    while True:
        response = yield symbol
        time.sleep(delay_s)


def safe_symbol(symbol, rate_limit):
    """
    Sends one *symbol* repeatedly obeying *rate_limit*
    """
    replenish_rate_s = 1.0 / rate_limit
    while True:
        for _ in range(rate_limit):
            response = yield symbol
            if response is Result.FAILURE:
                raise ValueError("Misconfiguration between client and server!")
            while True:
                time.sleep(replenish_rate_s)
                response = yield symbol
                if response is Result.FAILURE:
                    raise ValueError("Misconfiguration between client and server!")


def scenario(scenario):
    """
    Sends symbols according to *scenario*, where *scenario* is comma-separated
    string of symbols and delays in ms.
    """
    for item in scenario.split(","):
        if item.isnumeric():
            time.sleep(int(item) / 1000.0)
        else:
            response = yield item


def find_partitions(rate_limit):
    """
    Given *rate_limit* of a server, sends multiple requests to find out
    configured partitions.
    """
    replenish_rate_s = 1.0 / rate_limit
    print(f"Rate limit {rate_limit}; replenish rate {replenish_rate_s}s")
    start = time.time()
    found_partitions = [[SYMBOLS[0]]]
    for symbol in SYMBOLS[1:]:
        match = False
        print(f'Testing "{symbol}"')
        for partition in found_partitions:  # inner 1
            sample_element = partition[0]
            sample_response = yield sample_element
            while sample_response is not Result.FAILURE:
                sample_response = yield sample_element
            target_response = yield symbol
            time.sleep(replenish_rate_s)
            if target_response is Result.FAILURE:
                partition.append(symbol)
                match = True
                break  # inner
        if not match:
            found_partitions.append([symbol])
        time.sleep(1.0 - replenish_rate_s)
    took_ms = stamp_ms(start)
    print(f"Search took {took_ms}ms; found partitions:")
    for partition in found_partitions:
        print(sorted(partition))


def define_subcommand(subcommands, fn, args):
    subcmd = subcommands.add_parser(fn.__name__, help=fn.__doc__)
    subcmd.set_defaults(func=fn)
    for (arg_name, arg_props) in args.items():
        subcmd.add_argument(arg_name, **arg_props)


if __name__ == "__main__":
    parser = ArgumentParser(description="Request random symbols with given frequency")
    subcommands = parser.add_subparsers(help="Client mode", metavar="CMD", required=True)
    define_subcommand(subcommands,
                      fast_random,
                      dict(delay_ms=dict(metavar="DELAY",
                                         type=int,
                                         help="Delay between requests in ms")))
    define_subcommand(subcommands,
                      cycle,
                      dict(delay_ms=dict(metavar="DELAY",
                                         type=int,
                                         help="Delay between requests in ms")))
    define_subcommand(subcommands,
                      burst,
                      dict(iterations=dict(metavar="ITERATIONS",
                                           type=int,
                                           nargs="?",
                                           default=5,
                                           help="How many bursts to send")))
    define_subcommand(subcommands,
                      symbol,
                      dict(symbol=dict(metavar="SYMBOL",
                                       type=str,
                                       help="Symbol to send"),
                           delay_ms=dict(metavar="DELAY",
                                         type=int,
                                         help="Delay between requests in ms")))
    define_subcommand(subcommands,
                      safe_symbol,
                      dict(symbol=dict(metavar="SYMBOL",
                                       type=str,
                                       help="Symbol to send"),
                           rate_limit=dict(metavar="RATE_LIMIT",
                                           type=int,
                                           help="Rate limit configured on a server")))
    define_subcommand(subcommands,
                      scenario,
                      dict(scenario=dict(metavar="SCENARIO",
                                         type=str,
                                         help="Comma-separated list of commands and delays")))
    define_subcommand(subcommands,
                      find_partitions,
                      dict(rate_limit=dict(metavar="RATE_LIMIT",
                                           type=int,
                                           help="Rate limit configured on a server")))
    args = vars(parser.parse_args())
    func = args.pop("func")
    client = mk_client(*address())
    gen = func(**args)
    session(client, gen)
