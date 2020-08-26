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
  - Or do we just handle duplicate lists?
- Handle args in an order independent way
- Store DB file in .config/procast under \$HOME
- Implement a table printer

```
let table = table::new();
table.add_row(["ONE", "TWO", "THREE"]) // OK
table.add_row(["foo", "bar", "baz"]) // OK
table.add_row(["f", "b"]); // OK, empty string for last col
table.add_row(["a", "b", "c", "d"]); // Err, to many cols

table.print(std::out);
```

### ideas

- sync lists to the server on start up
  - use a history api to know what has changed since last sync
  - ensure sync works if db file is deleted
  - will need to resolve mismatched client/server db
