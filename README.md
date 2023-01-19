# task-changes

Show changes in a [Taskwarrior](https://taskwarrior.org/) database, similar to `git log`.

## Usage

    $ task-changes [-s, --short] [count]

If `count` is omitted, it shows all changes in the `undo.data` file.

Set `NO_COLOR` environment variable to `1` to disable colors in the output.

## Example

```console
$ task-changes 2
2021-03-24 22:55:56 [1] Euripidis fabulis delectari
  annotation_1616626556:
    + http://example.com: "data"
  modified:
    - 2021-03-24 22:55:26 +00:00 (2 days ago)
    + 2021-03-24 22:55:28 +00:00 (2 days ago)

2021-03-24 22:55:26 [2] Quasi concordia
  project: second
  description: Quasi concordia
  modified: 2021-03-24 22:55:28 +00:00 (2 days ago)
  status: pending
  uuid: e505f4ba-cb73-42a7-9301-a4b2c68533c9
  entry: 2021-03-24 22:55:28 +00:00 (2 days ago)
```
