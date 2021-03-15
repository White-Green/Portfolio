Param(
    [string]$Token = "-"
)

cargo run -p works_generator -- --username White-Green --info ./tools/works_generator/additional_information.json --data ./static/works.data.json --graph ./static/works.graph.dot --token $Token
dot -Tsvg -o./static/works.graph.svg ./static/works.graph.dot
