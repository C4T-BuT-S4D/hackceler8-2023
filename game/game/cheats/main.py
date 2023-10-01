import tkinter as tk
from settings import Settings
import time
from dataclasses import asdict, fields
import requests


def main():
    port = 5000

    window = tk.Tk()
    window.geometry("500x500")
    settings = Settings()

    params = {}

    for field in fields(Settings):
        param_label = tk.Label(text=field.name)
        param_label.pack()

        if field.type == str:
            variable = tk.StringVar()
            param = tk.Entry(textvariable=variable)
            params[field.name] = variable
            param.pack()
        elif field.type == bool:
            variable = tk.BooleanVar()
            param = tk.Checkbutton(variable=variable)
            params[field.name] = variable
            param.pack()
        else:
            assert False, f"invalid type: {field.type}"

    button = tk.Button(text="Update settings!")
    ok_label = tk.Label(text="_")

    button.pack()
    ok_label.pack()

    def update_settings(_):
        for param_name, param_var in params.items():
            setattr(settings, param_name, param_var.get())

        r = requests.post(f"http://localhost:{port}/update_settings", json=asdict(settings))
        if r.status_code == 200:
            ok_label.config(text=f"ok {time.time()}")
        else:
            ok_label.config(text=f"err {time.time()}")

    button.bind("<Button-1>", update_settings)

    window.mainloop()


if __name__ == "__main__":
    main()
