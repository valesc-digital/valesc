# TODO
IMPLEMENT $18/CLC.
ADD DRY.

- :#02X should be :#04X
- :#04X should be :#06X DUE TO the `#`.
- Modules to refactor:
    - ines.rs
- CHECK DISCORD AND FINISH BRANCH IF CARRY
- Implement more opcodes.
- Test the stack. (With opcodes?), move the functions to its file.
- Test address builder.
- Full parse iNES and test it.
- Make a real initialization step on the CPU (https://www.reddit.com/r/EmuDev/comments/g663hk/nestestlog_stack_pointer_starting_at_fd_and_sbc/).
- Fix OOB and remove NOP on tests.