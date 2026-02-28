from typing import Any, Callable, TypeVar, cast


F = TypeVar("F", bound=Callable[..., Any])


def typed_tool(server: Any) -> Callable[[F], F]:
    return cast(Callable[[F], F], server.tool())


def typed_resource(server: Any, uri: str) -> Callable[[F], F]:
    resource = getattr(server, "resource", None)
    if callable(resource):
        return cast(Callable[[F], F], resource(uri))
    return lambda func: func


def typed_prompt(server: Any) -> Callable[[F], F]:
    prompt = getattr(server, "prompt", None)
    if callable(prompt):
        return cast(Callable[[F], F], prompt())
    return lambda func: func
