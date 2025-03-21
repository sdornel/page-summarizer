# deep-research

Ideas and plans

- don't get IP banned
- respect robots.txt file
    - find rules and parse then dynamically
- use https://commoncrawl.org/
    - they don't want you to ddos them. be nice to their servers
        - crawl their data (somewhat slowly) with a bot and store results in your own database server?
            - fetch database results to the frontend?

- implement AI agents to summarize and analyze data

```
    Research Agent (Agent 1): The first agent uses Firecrawl to search multiple web sources, extract relevant content, and generate an initial research report. Think of this agent as your data gatherer and initial synthesizer.

    Elaboration Agent (Agent 2): The second agent takes the baton, analyzing the initial report and transforming it into a more comprehensive document with additional context, examples, case studies, and implications. This agent acts as your content enhancer and expert editor.

    Coordination Process: OpenAI's Agents SDK manages the handoff between these specialized agents, ensuring that each focuses on its specific strengths, similar to how a research team might have different specialists for data collection and analysis.
```