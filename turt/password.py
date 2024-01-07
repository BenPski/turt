#!/usr/bin/env python3

import random
import string

class RandChoice():
    def __init__(self, selection):
        self.selection = list(selection)

    def generate(self):
        i = random.randrange(0, len(self.selection))
        return self.selection[i]

    def join(self, other):
        return RandChoice(list(set(self.selection + other.selection)))

    def __radd__(self, other):
        if isinstance(other, RandChoice):
            return self.join(other)
        else:
            raise "Must add random choices together only"

class Generic(RandChoice):
    def __init__(self):
        super().__init__(string.ascii_letters + string.digits + string.punctuation)

class Alpha(RandChoice):
    def __init__(self):
        super().__init__(string.ascii_letters)

class Uppercase(RandChoice):
    def __init__(self):
        super().__init__(string.ascii_uppercase)

class Lowercase(RandChoice):
    def __init__(self):
        super().__init__(string.ascii_lowercase)

class Digit(RandChoice):
    def __init__(self):
        super().__init__(string.digits)

class Symbol(RandChoice):
    def __init__(self):
        super().__init__(string.punctuation)

# not really needed, but a nicer name
class Selection(RandChoice):
    def __init__(self, choices):
        super().__init__(choices)


# generate a password
# want to be able to provide some kind of constraints or rules that need to be
# followed
class Password():
    def __init__(self, length = 32, base = Generic(), required = None):
        self.length = length
        self.base = base
        if required is None:
            self.required = []
        else:
            self.required = required
        self.config = self.required + ([self.base] * (self.length - len(self.required)))

    def generate(self):
        random.shuffle(self.config)
        password = [i.generate() for i in self.config]
        return ''.join(password)


