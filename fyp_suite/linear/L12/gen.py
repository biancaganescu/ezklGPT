

"""
Reference: https://github.com/karpathy/nanoGPT
"""

import json
import math
from dataclasses import dataclass
import torch
import torch.nn as nn
from torch.nn import functional as F
import sys
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.append(os.path.dirname(SCRIPT_DIR))

class MLP(nn.Module):
    def __init__(self, dim):
        super(MLP, self).__init__()
        self.linear_layer = nn.Linear(dim, dim)
        self.relu = nn.ReLU()
        self.scalar = 2
    
    def forward(self, x):
        x = self.linear_layer(x)
        x = self.relu(x)
        x = x / self.scalar
        return x

class Model(nn.Module):
    def __init__(self, n_layers, dim):
        super(Model, self).__init__()
        self.layers = nn.ModuleList([MLP(dim) for _ in range(n_layers)])
         # init all weights
        self.apply(self._init_weights)
            # report number of parameters
        print("number of parameters: %.2fM" % (self.get_num_params()/1e6,))

    def forward(self, x):
        for layer in self.layers:
            x = layer(x)
        return x

    def get_num_params(self):
        return sum(p.numel() for p in self.parameters())

    def _init_weights(self, module):
        if isinstance(module, nn.Linear):
            torch.nn.init.normal_(module.weight, mean=0.0, std=0.02)
            if module.bias is not None:
                torch.nn.init.zeros_(module.bias)



model = Model(n_layers=12, dim=128)

model.get_num_params()


shape = [128]
x =  0.1*torch.rand(1, 128, requires_grad=True)
torch_out = model(x)

torch.onnx.export(model, x, "network.onnx",
                  export_params=True,        # store the trained parameter weights inside the model file
                  opset_version=10,          # the ONNX version to export the model to
                  do_constant_folding=True,  # whether to execute constant folding for optimization
                  input_names=['input'],   # the model's input names
                  output_names=['output'],  # the model's output names
                  dynamic_axes={'input': {0: 'batch_size'},    # variable length axes
                                'output': {0: 'batch_size'}})

d = ((x).detach().numpy()).reshape([-1]).tolist()

data = dict(input_shapes=[shape],
            input_data=[d],
            output_data=[((o).detach().numpy()).reshape([-1]).tolist() for o in torch_out])

# Serialize data into file:
json.dump(data, open("input.json", 'w'))