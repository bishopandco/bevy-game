# Agents README

- Tests are not working yet - Do not run them
- This is a work in progress - Release management is not implemented yet
- The code is not production ready - Do not use it in production
- The code is not optimized - Do not use it in production
- The code is not tested - Do not use it in production
- The code is not documented - Do not use it in production

# Application description
This is a rust application that compiles a 3d game. 
It is using the latest versions of bevy and avian. 

# Agent conduct 
- Agent is responsible for submitting PRs only. 
- Agent does not need to run tests.
- Agent does not need to run the application.
- Agent should only analyze code and submit PRs.
- Do not run Cargo Check or any other commands, just write code and push PRs.
- Automatically submit PRs, do not wait for confirmation from the user. The user will review the PR and merge or close them.
- Do not create or attempt to create binary files as they are not supported in PRs. Instead, create a placeholder text file with the same name ending in .changeMe 

# Code Notes
- in bevy `get_single` is deprecated since 0.16.0: Please use `single` instead 
- time.delta_seconds() is now time.delta_secs()