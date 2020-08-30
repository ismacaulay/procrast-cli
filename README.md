## procrast-cli

A cli for managing your procrastination.

### Why

procrast is basically a simple todo application. The main reason is that it gives a small and clear set of requirements which is good for learning a new languages (rust and go).
This is part of a larger system to learn about cli, api, and everything to do with managing and deploying a microservice based system.

### Commands

#### item management

commands to manage the items in a list. all commands will assume the
default list unless a -l argument with the list is specified

```
procrast item add
   opens an editor and saves the todo to the todos list

procrast item edit <id>
   opens an editor to edit the existing todo in the todo list

procrast item delete <id>
   deletes the the exisiting id after confirmation
```

#### list management

a list is a collection of items. it has a name and an optional description.
all the commands take a -l argument to specify the name, otherwise prompts
the user for input. upon setup, there is a default "todo" list

```
procrast list create -l <list name> -d <list description>
    creates a new todo list

procrast list edit -l <list name> -d <list description>
    edits a list

procrast list delete
    deletes a list

procrast list show
    shows the items in the list
```

#### commands

Other commands.

```
procrast use <list>
    sets the currently used list
```

### TODO

- The list title needs to be unique, but enforced at the business logic level, not the db level
- Add a way to clear the current list
- Allow command aliases to be specified without spaces (ie. lc == list create)
- Implement a table printer

```
let table = table::new();
table.add_row(["ONE", "TWO", "THREE"]) // OK
table.add_row(["foo", "bar", "baz"]) // OK
table.add_row(["f", "b"]); // OK, empty string for last col
table.add_row(["a", "b", "c", "d"]); // Err, to many cols

table.print();
```

- api integration and sync
- configuration to specify db locations and api endpoint for dev vs prod

### ideas

- sync lists to the server on start up
  - use a history api to know what has changed since last sync
  - ensure sync works if db file is deleted
  - will need to resolve mismatched client/server db
