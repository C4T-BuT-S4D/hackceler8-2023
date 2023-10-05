from threading import Condition
from threading import RLock

__lock: RLock = RLock()
__cond: Condition = Condition(__lock)
__render_requested: bool = False
__render_finished: bool = False


def render_requested() -> bool:
    with __lock:
        return __render_requested


def request_render():
    global __render_requested, __render_finished
    with __cond:
        if __render_requested or __render_finished:
            # wait for other request_render to finish
            __cond.wait_for(lambda: not __render_requested and not __render_finished)

        # only one request_render here at a time
        __render_requested = True
        __render_finished = False

        __cond.wait_for(lambda: __render_finished)

        __render_requested = False
        __render_finished = False
        __cond.notify_all()


def render_finish() -> bool:
    global __render_requested, __render_finished
    with __lock:
        __render_requested = False
        __render_finished = True
        __cond.notify_all()
