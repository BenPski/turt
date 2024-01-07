import os
import getpass
from cryptography.fernet import Fernet
import cryptography
import base64
import json
import hashlib
from password import Password
from errors import *

class Vault():
    """
    The actual vault, it is an encrypted file
    To work with it you need to load the data by giving it a key which is
    derived from the password
    """
    def __init__(self, directory, password):
        self.directory = directory
        self.location = os.path.join(self.directory, "vault.turt")
        self.entries_location = os.path.join(self.directory, "entries.json")
        self.fer = Fernet(self._key(password))
        os.makedirs(os.path.dirname(self.location), exist_ok=True)
        if not os.path.exists(self.location) or not os.path.exists(self.entries_location):
            self.data = Vault.load_data({})
            print("Creating initial vault")
            self._save()

        self.data = self._load()

    def _key(self, password):
        return base64.urlsafe_b64encode(hashlib.scrypt(password.encode(), salt=b'salt', n=16, r=8, p=1, dklen=32))

    def _load(self):
        with open(self.location, 'rb') as f:
            data = f.read()
        try:
            as_json = json.loads(self.fer.decrypt(data))
            return Vault.load_data(as_json)
        except cryptography.fernet.InvalidToken:
            raise IncorrectPassword("Incorrect password")

    def _save(self):
        as_json = Vault.dump_data(self.data)
        with open(self.location, 'wb') as f:
            to_write = self.fer.encrypt(json.dumps(as_json).encode())
            f.write(to_write)
        entries = list(self.data.keys())
        with open(self.entries_location, 'w') as f:
            json.dump(entries, f)
   
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self._save()
        return False # don't supress exceptions

    # convert json to vault data
    @staticmethod
    def load_data(data):
        res = {}
        for (name, info) in data.items():
            res[name] = VaultData.load(info)
        return res

    # convert vault data to json
    @staticmethod
    def dump_data(data):
        res = {}
        for (name, info) in data.items():
            res[name] = info.to_dict()
        return res

    # Operations

    # general info
    def info(self):
        res = {}
        for (name, info) in self.data.items():
            x = {'type': info.data_type }
            x.update(info.properties)
            res[name] = x
        return res

    # get a particular piece of data
    # useful if a particular kind of data implements a special action
    def grab(self, item):
        return self.data.get(item)

    # get the value for the entry
    def get(self, item):
        vd = self.grab(item)
        if vd is None:
            return None
        else:
            return vd.get()

    # update the entry with a particular value 
    def update(self, item, value):
        vd = self.grab(item)
        if vd is not None:
            vd.update(value)
            return vd
        else:
            return None

    # create a new entry
    def create(self, item, data_type, value, properties):
        self.data[item] = VaultData.create(data_type, value, properties)
        return self.data[item]

    # delete an entry
    def delete(self, item):
        self.data.pop(item, None)


class VaultData():
    def __init__(self, data_type, value, properties):
        self.data_type = data_type
        self.value = value
        self.properties = properties

    def get(self):
        return self.value

    def update(self, val):
        self.value = val

    @staticmethod
    def create(data_type, value, properties):
        if data_type == 'password':
            return PasswordData(data_type, value, properties)
        elif data_type == 'credential':
            return CredentialData(data_type, value, properties)
        else:
            return GenericData(data_type, value, properties)

    @staticmethod
    def load(info):
        if info['type'] == 'password':
            return PasswordData.from_dict(info)
        elif info['type'] == 'credential':
            return CredentialData.from_dict(info)
        else:
            return GenericData.from_dict(info)

    @classmethod
    def from_dict(cls, info):
        return cls(info['type'], info['value'], info['properties'])

    def to_dict(self):
        out = {}
        out['type'] = self.data_type
        out['value'] = self.value
        out['properties'] = self.properties
        return out


class PasswordData(VaultData):
    def __init__(self, data_type, value, properties):
        self.data_type = data_type
        self.value = value
        self.properties = properties
        if 'length' not in self.properties:
            self.properties['length'] = 32
        self.length = self.properties['length']
        self.password = Password()

    def rotate(self):
        new = self.password.generate()
        self.update(new)

class CredentialData(VaultData):
    def __init__(self, data_type, value, properties):
        self.data_type = data_type
        self.value = value
        self.username = self.value['username']
        self.password = self.value['password']
        self.properties = properties
        if 'length' not in self.properties:
            self.properties['length'] = 32
        self.length = self.properties['length']
        self.password_gen = Password()

    def rotate(self):
        new = self.password_gen.generate()
        self.update({'username': self.username, 'password': new})

# just a catch all, json valued data
class GenericData(VaultData):
    pass
