import argparse
import tkinter as tk
import time
from dataclasses import asdict, fields

import requests

from settings import Settings


def main(port: int = 5000):
    window = tk.Tk()
    window.geometry("500x500")
    settings = Settings()

    params = {}

    for field in fields(Settings):
        param_label = tk.Label(text=field.name)
        param_label.pack()

        print(f"{field.name=} {field.type.__name__=}")
        match field.type.__name__:
            case "str" | "list":
                variable = tk.StringVar()

                if isinstance(getattr(settings, field.name), list):
                    variable.set(",".join(getattr(settings, field.name)))
                else:
                    variable.set(getattr(settings, field.name))

                param = tk.Entry(textvariable=variable)
                params[field.name] = variable
                param.pack()
            case "bool":
                variable = tk.BooleanVar()
                variable.set(getattr(settings, field.name))
                param = tk.Checkbutton(variable=variable)
                params[field.name] = variable
                param.pack()
            case "int":
                variable = tk.IntVar()
                variable.set(getattr(settings, field.name))
                param = tk.Entry(textvariable=variable)
                params[field.name] = variable
                param.pack()
            case _:
                assert False, f"invalid type: {field.type}"

    button = tk.Button(text="Update settings!")
    ok_label = tk.Label(text="_")

    button.pack()
    ok_label.pack()

    def update_settings(_):
        for param_name, param_var in params.items():
            val = param_var.get()
            if isinstance(
                next(filter(lambda x: x.name == param_name, fields(settings))), list
            ):
                assert isinstance(val, str), f"invalid type: {type(val)}"
                val = val.split()
            setattr(settings, param_name, val)

        r = requests.post(
            f"http://localhost:{port}/update_settings", json=asdict(settings)
        )
        if r.status_code == 200:
            ok_label.config(text=f"ok {time.time()}")
        else:
            ok_label.config(text=f"err {time.time()}")

    button.bind("<Button-1>", update_settings)

    window.mainloop()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog="cheats")
    parser.add_argument(
        "p", nargs="?", type=int, default=8889, help="Cheats server port"
    )

    args = parser.parse_args()
    main(port=args.p)
