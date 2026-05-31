1. Create a new git branch named <feature-slug>-<issue-number> before starting work.
2. Find the highest-priority feature to work on and work only on that feature.
This should be the one YOU decide has the highest priority - not necessarily the first in the list.
3. Check that the project compiles via cargo check and that the tests pass via cargo test.
4. Update the PRD with the work that was done.
5. Append your progress to the progress.txt file.
Use this to leave a note for the next person working in the codebase.
6. Make a git commit of that feature.
ONLY WORK ON A SINGLE FEATURE.
If, while implementing the feature, you notice the PRD is complete, output <promise>COMPLETE</promise>.
