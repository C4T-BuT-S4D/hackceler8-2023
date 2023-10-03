from flask import Flask, request
from cheats.settings import update_settings, Settings
import threading


def run_cheats_server(port: int) -> threading.Thread:
    app = Flask(__name__)

    @app.route("/update_settings", methods=["POST"])
    def update_settings_route():
        d = request.get_json()
        assert type(d) == dict

        def upd(settings: Settings):
            for param_name, param_value in d.items():
                assert type(param_name) == str
                assert hasattr(settings, param_name)
                setattr(settings, param_name, param_value)

        update_settings(upd)
        return "Hello, World!"

    t = threading.Thread(target=lambda: app.run(host="localhost", port=port))
    t.start()
