import os
import shutil
import json
from constants import *
from vault import Vault
from errors import VaultExists, VaultDoesNotExist

class VaultManager():
    """
    Interface around multiple vaults
    Each vault is stored in a separate directory
    Can't make any direct updates to vaults only retrieve them
    """
    def __init__(self):
        self.directory = DIRECTORY
        os.makedirs(self.directory, exist_ok=True)
        self.config_location = os.path.join(self.directory, 'config.json')
        if not os.path.exists(self.config_location):
            self.data = self._default_config()
            self._save()
        else:
            self.data = self._load()

    # get all the sub directories
    def _dirs(self):
        return [d for d in os.scandir(self.directory) if d.is_dir()]

    def _default_config(self):
        data = {
                'port': 5490,
                }
        return data
    
    def _save(self):
        with open(self.config_location, 'w') as f:
            json.dump(self.data, f, sort_keys=True, indent=4)

    def _load(self):
        with open(self.config_location, 'r') as f:
            return json.load(f)

    # list of the available vaults
    def available(self):
        return list(map(lambda x: x.name, self._dirs()))

    # attempt to fill in label when one isn't provided
    def get_label(self, label=None):
        if label is None and len(self.available()) == 1:
            return self.available()[0]
        elif label is None and len(self.available()) != 1:
            raise ArgumentError("Need to provide a label since there is more than 1 vault to choose from")
        elif label in self.available():
            return label
        else:
            raise VaultDoesNotExist(f"Can't get {label} it doesn't exist")

    # check entries of a vault
    def entries(self, label=None):
        label = self.get_label(label)
        with open(os.path.join(self.directory, label, "entries.json"), 'r') as f:
            data = json.load(f)
        return data

    # initialize a new vault with a label
    def create(self, label, password):
        if label not in self.available():
            os.mkdir(os.path.join(self.directory, label))
            return Vault(os.path.join(self.directory, label), password)
        else:
            raise VaultExists(f"Can't create {label} it already exists")

    # delete a vault
    def delete(self, label):
        if label in self.available():
            shutil.rmtree(os.path.join(self.directory, label))
        else:
            raise VaultDoesNotExist(f"Can't delete {label} it doesn't exist")

    # rename a vault
    def rename(self, label, new_label):
        pass

    # get a vault
    def get(self, password, label=None):
        label = self.get_label(label)
        return Vault(os.path.join(self.directory, label), password)

