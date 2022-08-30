#!/usr/bin/env python3

import sys
import string
import random


def first(s):
    return next(iter(s))


def partitions(inp, k):
    shuffled = list(inp)
    random.shuffle(shuffled)

    if k == 1:
        return [shuffled]
    elif k == len(inp):
        return [[x] for x in shuffled]

    ixs = list(range(len(inp) - 1))
    random.shuffle(ixs)
    splits = sorted(ixs[:k-1])

    bins = []
    last = 0
    for split in splits:
        bins.append(shuffled[last:split])
        last = split
    bins.append(shuffled[split:])
    return sorted((frozenset(b) for b in bins), key=hash)


class MockService:
    def __init__(self, partitions):
        self._table = {
            k:g for g in partitions for k in g
        }

    def are_in_the_same_group(self, a, b):
        a_group = self._table[a]
        return b in a_group


def main(k):
    iset = string.ascii_uppercase
    ps = partitions(iset, k)
    print('Generated {} groups:'.format(len(ps)))
    for p in ps:
        print(p)
    ms = MockService(ps)
    print('Starting reverse engineering process...')
    found_groups = [{iset[0]}]
    for symbol in iset[1:]:
        match = False
        for g in found_groups: # inner
            sample_el = first(g)
            if ms.are_in_the_same_group(symbol, sample_el):
                g.add(symbol)
                match = True
                break # inner
        if not match:
            found_groups.append({symbol})

    result = sorted((frozenset(fg) for fg in found_groups), key=hash)

    print('Found {} groups:'.format(len(result)))
    for r in result:
        print(r)


if __name__ == '__main__':
    if len(sys.argv) <= 1:
        print('Usage: ./prototype.py K [SEED]', file=sys.stderr)
        sys.exit(1)
    if len(sys.argv) > 2:
        seed = sys.argv[2]
        random.seed(seed)
    k = int(sys.argv[1])
    main(k)
