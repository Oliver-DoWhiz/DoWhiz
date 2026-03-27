const EN_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz | Meet Oliver, your AI operator',
    description:
      'Meet Oliver, the trusted AI operator for your work and life. Connect the tools you already use, give him one real task, review the result, and turn repeat work into a routine.',
    canonicalUrl: 'https://dowhiz.com/',
    ogLocale: 'en_US',
    themeColor: '#2C2C2E',
    htmlLang: 'en'
  },
  nav: {
    homePath: '/',
    links: [
      { href: '#getting-started', label: 'Start here' },
      { href: '#use-cases', label: 'What Oliver does' },
      { href: '#surfaces', label: 'Where Oliver works' },
      { href: '#control', label: 'Review & control' },
      { href: '#faq', label: 'FAQ' },
      { href: '#blog', label: 'Blog' }
    ],
    signIn: 'Sign in',
    dashboard: 'Oliver setup',
    signOut: 'Sign out',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: 'Contact Oliver'
  },
  hero: {
    eyebrow: 'Meet Oliver',
    title: 'Oliver is the AI operator for your work and life.',
    subtitle:
      'Tell Oliver what needs doing. He works in your tools, returns finished work, and keeps the next step clear.',
    note: 'Start with one connection and one real task. No giant new system required.',
    chips: ['Email', 'Slack', 'Discord', 'GitHub', 'Notion', 'Lark'],
    primaryCta: 'Open your setup',
    secondaryCta: 'See what Oliver can handle',
    secondaryHref: '#use-cases',
    contactSubject: 'Talk to me about Oliver',
    contactBody:
      'Hi Oliver,\n\nI want to see whether DoWhiz fits the following work or life tasks:\n-\n-\n-\n\nThanks!',
    onboardingAriaLabel: 'Oliver onboarding steps',
    onboardingTitle: 'Start in four simple steps',
    onboardingDescription:
      'The first success should feel personal and useful, not like you are deploying a whole new system.',
    profileEyebrow: 'Trusted AI operator',
    profileTitle: 'Oliver works where you already are',
    profileDescription:
      'Use Oliver for inbox work, research, drafting, coordination, and follow-ups without rebuilding your workflow.',
    profilePoints: [
      'Connect only the apps you want Oliver to use',
      'Review drafts before sensitive actions',
      'Keep your tone, preferences, and context in one shared memory'
    ],
    exampleTitle: 'Good first tasks',
    exampleTasks: [
      'Summarize this topic and give me the trade-offs in one page.',
      'Draft a reply to this email and pull the action items into Notion.',
      'Organize these tax documents and tell me what is still missing.',
      'Draft next week’s LinkedIn post and hold it for my approval.'
    ]
  },
  sections: {
    gettingStartedTitle: 'How to get started',
    gettingStartedIntro: 'The first win should feel small, concrete, and easy to review.',
    useCasesTitle: 'What Oliver can take off your plate',
    useCasesIntro: 'Start with the tasks you already repeat, postpone, or keep half-finishing.',
    surfacesTitle: 'Where Oliver works',
    surfacesIntro: 'Oliver shows up in the places where requests already land and where work already lives.',
    controlTitle: 'Review and control',
    controlIntro: 'You decide what Oliver can access, where he can work, and when a sensitive action should wait for approval.',
    faqTitle: 'Questions people ask before they trust an AI operator',
    faqIntro: 'The answers focus on control, clarity, and a realistic first-run experience.'
  },
  onboardingSteps: [
    {
      id: '01',
      title: 'Connect one tool',
      desc: 'Start with the place where work already lands, like email, Notion, Slack, or GitHub.',
      detail: 'You do not need to connect everything before Oliver is useful.'
    },
    {
      id: '02',
      title: 'Give Oliver a first task',
      desc: 'Pick one real task that already exists in your life or work.',
      detail: 'A good first task is specific, reviewable, and annoying enough that you do not want to do it twice.'
    },
    {
      id: '03',
      title: 'Review the result',
      desc: 'Check the draft, summary, note, or follow-up before it goes anywhere important.',
      detail: 'This is how you learn what Oliver should own, what should stay manual, and what needs approval.'
    },
    {
      id: '04',
      title: 'Turn repeat work into a routine',
      desc: 'Once a task works, reuse it, save preferences, and keep future follow-ups faster.',
      detail: 'DoWhiz starts personal first, then grows into recurring work and shared use when you are ready.'
    }
  ],
  useCases: [
    {
      tag: 'Deep research',
      title: 'Turn open questions into decision-ready research',
      desc:
        'Ask Oliver to compare options, gather sources, and return a concise brief you can actually use.'
    },
    {
      tag: 'Inbox',
      title: 'Draft replies, triage requests, and keep follow-ups moving',
      desc:
        'Oliver can read the context you share, draft the response, and keep the next action from falling through.'
    },
    {
      tag: 'Writing support',
      title: 'Support research, outlining, revision, and structured drafting',
      desc:
        'Use Oliver for study support, knowledge work, and document cleanup without turning the experience into chat-for-chat’s-sake.'
    },
    {
      tag: 'Tax prep',
      title: 'Prepare documents and organize what a filing still needs',
      desc:
        'Oliver can sort inputs, summarize gaps, and get materials ready for review before you file.'
    },
    {
      tag: 'Content',
      title: 'Draft and schedule posts with your approval',
      desc:
        'Let Oliver prepare LinkedIn copy, shape a posting cadence, and queue drafts for a final human yes.'
    },
    {
      tag: 'GitHub',
      title: 'Follow issues, code context, and repo-side tasks',
      desc:
        'Oliver can help keep engineering work moving by organizing issue context, summaries, and code-adjacent follow-ups.'
    }
  ],
  surfaces: [
    {
      tag: 'Email',
      title: 'Email and personal admin',
      desc:
        'Handle requests, draft replies, organize document-heavy tasks, and keep follow-up work moving from your inbox.',
      icon: '/icons/The%20Digital%20Employee%20Stack/trigger.svg'
    },
    {
      tag: 'Slack / Discord',
      title: 'Chat where work actually happens',
      desc:
        'Use Oliver in the conversations where people already ask for help, share decisions, and need quick follow-through.',
      icon: '/icons/The%20Digital%20Employee%20Stack/execute.svg'
    },
    {
      tag: 'GitHub',
      title: 'Repo context and execution handoffs',
      desc:
        'Keep issues, reviews, and code-related requests anchored in the same place your engineering work already lives.',
      icon: '/icons/The%20Digital%20Employee%20Stack/shared.svg'
    },
    {
      tag: 'Notion',
      title: 'Notes, plans, and knowledge that stay usable',
      desc:
        'Turn raw notes into reusable pages, concise updates, and shared references that do not get lost in chat.',
      icon: '/icons/The%20Digital%20Employee%20Stack/agent.svg'
    },
    {
      tag: 'Shared docs',
      title: 'Drafting and structured document workflows',
      desc:
        'Use Oliver for Google Docs style workflows, document revision, and formal deliverables that need to stay readable and reviewable.',
      icon: '/icons/The%20Digital%20Employee%20Stack/collaboration.svg'
    },
    {
      tag: 'Lark',
      title: 'Cross-team coordination when you need it',
      desc:
        'Start as a personal operator today, then extend Oliver into shared SMB surfaces when the workflow proves itself.',
      icon: '/icons/The%20Digital%20Employee%20Stack/permission.svg'
    }
  ],
  safetyItems: [
    {
      tag: 'Control',
      title: 'You control what Oliver can access',
      icon: '/icons/key.svg',
      desc:
        'Only connect the apps and surfaces you actually want Oliver to use, and remove access whenever you want.',
      points: [
        'Access is explicit and revocable',
        'You can start with one tool instead of a full rollout',
        'Sensitive actions can stay behind a review step'
      ]
    },
    {
      tag: 'Review',
      title: 'You can review before important actions',
      icon: '/icons/shield_lock.svg',
      desc:
        'Use Oliver for drafts, summaries, research, and prep work first, then increase trust step by step.',
      points: [
        'Drafts can be checked before sending',
        'Complex tasks can return a structured result instead of acting blindly',
        'You decide which routines are ready to repeat'
      ]
    },
    {
      tag: 'Identity',
      title: 'No personal credential handoff by default',
      icon: '/icons/lock_person.svg',
      desc:
        'DoWhiz is built around controlled access and operator-style execution, not around asking you to give away personal passwords.',
      points: [
        'Agent-owned identity model where applicable',
        'Less credential sprawl across ad hoc tools',
        'Clearer boundaries for personal and shared work'
      ]
    }
  ],
  accessFlowSteps: [
    {
      title: 'Connect',
      desc: 'Choose the first app or surface where Oliver should work.'
    },
    {
      title: 'Scope',
      desc: 'Keep the initial task narrow enough that you can review it quickly.'
    },
    {
      title: 'Review',
      desc: 'Check the result, adjust your preferences, and decide whether the task is ready to repeat.'
    },
    {
      title: 'Expand',
      desc: 'Only after the first task works should you add more apps, more automation, or shared SMB workflows.'
    }
  ],
  faqItems: [
    {
      question: 'What is DoWhiz now?',
      answer:
        'DoWhiz starts with Oliver: a trusted AI operator who works in your tools and brings back finished work you can review and reuse.'
    },
    {
      question: 'How should I start?',
      answer:
        'Connect one tool, give Oliver one real task, review the result, then save the preferences that made it useful.'
    },
    {
      question: 'Does Oliver remember context?',
      answer:
        'Yes. Oliver can keep shared memory for your preferences, task history, and follow-ups so you do not have to restart from zero each time.'
    },
    {
      question: 'Do I need to hand over my passwords?',
      answer:
        'No. DoWhiz is designed around controlled access and reviewable execution instead of asking for broad personal credential handoff.'
    },
    {
      question: 'Can Oliver help with both personal and work tasks?',
      answer:
        'Yes, as long as the task is clear, reviewable, and appropriate. Good examples include research, drafting, document organization, follow-ups, and tool-native task execution.'
    },
    {
      question: 'Is DoWhiz only for individuals?',
      answer:
        'The product starts with personal usefulness first. Once Oliver is trusted in one person’s workflow, the same setup can grow into shared SMB routines later.'
    }
  ],
  blogPosts: [
    {
      tag: 'Workflow guide',
      title: 'AI workflow automation checklist for lean teams',
      date: 'March 27, 2026',
      excerpt: 'A practical checklist for triggers, quality gates, and reliable delivery across the tools people already use.',
      link: '/blog/ai-workflow-automation-checklist/'
    },
    {
      tag: 'Engineering',
      title: 'GitHub issue automation best practices',
      date: 'February 26, 2026',
      excerpt: 'A steadier issue-to-PR model with better scoping, validation, and reviewer-ready handoffs.',
      link: '/blog/github-issue-automation-best-practices/'
    },
    {
      tag: 'Email',
      title: 'Email task automation playbook for operations teams',
      date: 'February 26, 2026',
      excerpt: 'How to turn inbox requests into structured execution, progress updates, and finished deliverables.',
      link: '/blog/email-task-automation-playbook/'
    },
    {
      tag: 'Memory',
      title: 'AI agent memory best practices',
      date: 'February 26, 2026',
      excerpt: 'What to remember, what to reset, and how to keep cross-channel follow-ups useful instead of noisy.',
      link: '/blog/ai-agent-memory-best-practices/'
    }
  ],
  labels: {
    blogEyebrow: 'From the blog',
    blogTitle: 'Notes on building useful operator-style automation',
    blogIntro: 'Practical writing about memory, inbox work, automation quality, and where AI operators break down in the real world.',
    blogHeaderButton: 'View all posts',
    blogLinkLabel: 'Read article',
    accessPlaybookTitle: 'How Oliver setup expands',
    accessPlaybookDescription:
      'You do not need to do everything at once. Start narrow, prove value, then add the next tool or repeated workflow when it is earned.',
    accessPlaybookLink: 'Explore Trust & Safety',
    faqCta: 'Open the full Help Center',
    footerTitle: 'Explore',
    footerTagline: 'Oliver helps turn requests into finished work, one connected tool and one reviewed task at a time.',
    footerPill: 'Personal first. Shared workflows later.',
    footerBottomSecondary: 'Built for clear onboarding, controlled execution, and real work.'
  },
  footerLinks: [
    { href: '/privacy/', label: 'Privacy' },
    { href: '/terms/', label: 'Terms of Service' },
    { href: '/trust-safety/', label: 'Trust & Safety' },
    { href: '/integrations/', label: 'Integrations' },
    { href: '/user-guide/', label: 'User Guide' },
    { href: 'https://www.dowhiz.com/help-center/', label: 'Help Center' },
    { href: '/solutions/ai-workflow-automation/', label: 'AI Workflow Automation' },
    { href: '/solutions/github-issue-automation/', label: 'GitHub Issue Automation' },
    { href: '/solutions/slack-task-automation/', label: 'Slack Task Automation' },
    { href: '/solutions/email-task-automation/', label: 'Email Task Automation' },
    { href: '/solutions/google-docs-automation/', label: 'Google Docs Automation' }
  ]
};

const ZH_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz 中文 | 认识 Oliver，你的 AI operator',
    description:
      'Oliver 是一个为工作和生活服务的 AI operator。连接你已经在用的工具，先交给他一个真实任务，再逐步建立你的个人工作流。',
    canonicalUrl: 'https://dowhiz.com/cn',
    ogLocale: 'zh_CN',
    themeColor: '#2C2C2E',
    htmlLang: 'zh-CN'
  },
  nav: {
    homePath: '/cn',
    links: [
      { href: '#getting-started', label: '开始方式' },
      { href: '#use-cases', label: 'Oliver 能做什么' },
      { href: '#surfaces', label: 'Oliver 在哪里工作' },
      { href: '#control', label: '审阅与控制' },
      { href: '#faq', label: '常见问题' },
      { href: '#blog', label: '博客' }
    ],
    signIn: '登录',
    dashboard: 'Oliver 设置',
    signOut: '退出登录',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: '联系 Oliver'
  },
  hero: {
    eyebrow: '认识 Oliver',
    title: 'Oliver 是一个服务你工作和生活的 AI operator。',
    subtitle:
      '告诉 Oliver 你要完成什么。他会在你已有的工具里工作，把结果带回来，并帮你把下一步变清楚。',
    note: '先从一个连接、一个真实任务开始，不需要先搭新的大系统。',
    chips: ['Email', 'Slack', 'Discord', 'GitHub', 'Notion', 'Lark'],
    primaryCta: '打开我的设置',
    secondaryCta: '看看 Oliver 能处理什么',
    secondaryHref: '#use-cases',
    contactSubject: '想了解 Oliver 是否适合我的场景',
    contactBody:
      '你好 Oliver，\n\n我想看看 DoWhiz 是否适合下面这些工作或生活任务：\n-\n-\n-\n\n谢谢！',
    onboardingAriaLabel: 'Oliver onboarding 步骤',
    onboardingTitle: '四步上手',
    onboardingDescription: '第一次成功最好是小而具体、容易审阅，而不是一上来就要部署整套系统。',
    profileEyebrow: '可信的 AI operator',
    profileTitle: 'Oliver 在你已经在用的地方工作',
    profileDescription:
      '你可以把收件箱、研究、写作、整理、跟进这些事情交给 Oliver，而不用先改造自己的工作方式。',
    profilePoints: [
      '只连接你想让 Oliver 使用的工具',
      '敏感动作前可以先看草稿再确认',
      '把你的语气、偏好和上下文放进同一份记忆里'
    ],
    exampleTitle: '适合先试的任务',
    exampleTasks: [
      '把这个主题做成一页研究简报，并写清利弊。',
      '根据这封邮件起草回复，并把待办同步到 Notion。',
      '帮我整理这些报税材料，并告诉我还缺什么。',
      '起草下周的 LinkedIn 帖子，等我确认后再发。'
    ]
  },
  sections: {
    gettingStartedTitle: '怎么开始',
    gettingStartedIntro: '第一次成功应该是清楚、具体、能快速复核的。',
    useCasesTitle: 'Oliver 可以接过哪些事情',
    useCasesIntro: '从那些你反复在做、总被打断、或者一直拖着没完成的任务开始。',
    surfacesTitle: 'Oliver 在哪里工作',
    surfacesIntro: '请求本来就落在哪里，工作本来就发生在哪里，Oliver 就应该在那里出现。',
    controlTitle: '审阅与控制',
    controlIntro: '你来决定 Oliver 能访问什么、在哪些地方工作、哪些动作必须等你确认。',
    faqTitle: '信任一个 AI operator 之前最常见的问题',
    faqIntro: '重点不是神奇，而是控制感、清晰度和第一次使用是否靠谱。'
  },
  onboardingSteps: [
    {
      id: '01',
      title: '先连接一个工具',
      desc: '从任务本来就会出现的地方开始，比如邮件、Notion、Slack 或 GitHub。',
      detail: '不用先把所有工具都接上，先让 Oliver 在一个真实场景里证明自己。'
    },
    {
      id: '02',
      title: '交给 Oliver 一个真实任务',
      desc: '选一个你本来就要做、而且结果容易检查的任务。',
      detail: '第一步最好足够具体，这样你能很快判断 Oliver 哪些事能接，哪些事还需要你亲自把关。'
    },
    {
      id: '03',
      title: '先看结果，再决定怎么继续',
      desc: '先看草稿、总结、整理结果或跟进建议，再决定要不要继续放行。',
      detail: '你会慢慢知道 Oliver 适合帮你做什么，也会知道哪些动作要保留审核。'
    },
    {
      id: '04',
      title: '把重复工作变成惯例',
      desc: '当某件事跑通以后，再去保存偏好、复用流程、或者扩展到更多工具。',
      detail: 'DoWhiz 先服务个人，再逐步扩展到共享的 SMB 工作流。'
    }
  ],
  useCases: [
    {
      tag: 'Deep research',
      title: '把模糊问题整理成可以决策的研究结果',
      desc: '让 Oliver 去比较方案、收集信息、提炼重点，最后给你一份可以直接使用的简报。'
    },
    {
      tag: 'Inbox',
      title: '起草回复、分拣请求、推进跟进',
      desc: 'Oliver 可以结合你给出的上下文写回复、整理下一步，让请求不再卡在收件箱里。'
    },
    {
      tag: 'Writing support',
      title: '支持研究、提纲、改写和结构化写作',
      desc: '适合学习支持、知识型工作和文稿整理，不把体验做成单纯聊天。'
    },
    {
      tag: 'Tax prep',
      title: '做报税准备和材料整理',
      desc: 'Oliver 可以帮你分类文件、指出缺失项、把提交前需要看的内容先整理清楚。'
    },
    {
      tag: 'Content',
      title: '起草并排期内容，发布前由你确认',
      desc: '让 Oliver 准备 LinkedIn 文案、安排节奏、生成草稿，最后由你点头。'
    },
    {
      tag: 'GitHub',
      title: '跟进 issue、代码上下文和 repo 内事务',
      desc: 'Oliver 可以帮你整理 issue、追踪上下文、推进和代码相关的协作任务。'
    }
  ],
  surfaces: [
    {
      tag: 'Email',
      title: '邮件和个人行政事务',
      desc: '从收件箱里的请求、材料整理、回复草稿和后续跟进开始。',
      icon: '/icons/The%20Digital%20Employee%20Stack/trigger.svg'
    },
    {
      tag: 'Slack / Discord',
      title: '在聊天里接任务、回结果',
      desc: '工作本来就在这些对话里发生，Oliver 也应该直接在这里接住它。',
      icon: '/icons/The%20Digital%20Employee%20Stack/execute.svg'
    },
    {
      tag: 'GitHub',
      title: '在代码库上下文里推进事情',
      desc: 'issue、review、工程协作都尽量留在同一个地方，不要来回复制粘贴。',
      icon: '/icons/The%20Digital%20Employee%20Stack/shared.svg'
    },
    {
      tag: 'Notion',
      title: '把笔记、计划和知识整理成能继续用的东西',
      desc: '把零散信息变成页面、更新和共享参考，而不是散在聊天记录里。',
      icon: '/icons/The%20Digital%20Employee%20Stack/agent.svg'
    },
    {
      tag: 'Shared docs',
      title: '文档起草与正式材料整理',
      desc: '适合 Google Docs 一类的写作、修订和正式交付场景。',
      icon: '/icons/The%20Digital%20Employee%20Stack/collaboration.svg'
    },
    {
      tag: 'Lark',
      title: '准备好以后再扩展到团队协作',
      desc: '先把个人流程跑通，再把 Oliver 扩展到共享的 SMB 场景。',
      icon: '/icons/The%20Digital%20Employee%20Stack/permission.svg'
    }
  ],
  safetyItems: [
    {
      tag: '控制',
      title: '你来决定 Oliver 能访问什么',
      icon: '/icons/key.svg',
      desc: '只连接你真的想让 Oliver 使用的工具，权限也能随时撤回。',
      points: [
        '权限明确、可撤销',
        '可以先从一个工具开始',
        '敏感动作前可以保留确认步骤'
      ]
    },
    {
      tag: '审阅',
      title: '重要动作前先看结果',
      icon: '/icons/shield_lock.svg',
      desc: '先把 Oliver 用在草稿、总结、研究和准备工作上，再逐步扩大信任范围。',
      points: [
        '先看草稿再发送',
        '复杂任务先返回结构化结果',
        '你来决定哪些流程可以变成惯例'
      ]
    },
    {
      tag: '身份',
      title: '默认不需要交出个人密码',
      icon: '/icons/lock_person.svg',
      desc: 'DoWhiz 更强调受控访问和可审阅执行，而不是让你把个人账户完全交出去。',
      points: [
        '适用场景下使用 agent-owned identity',
        '减少凭证四处扩散',
        '个人使用和共享使用的边界更清晰'
      ]
    }
  ],
  accessFlowSteps: [
    {
      title: '连接',
      desc: '先选 Oliver 真正要工作的第一个入口。'
    },
    {
      title: '收窄范围',
      desc: '第一次任务尽量足够具体，方便快速审核。'
    },
    {
      title: '审阅',
      desc: '先看结果，调整偏好，再判断这件事是否适合重复。'
    },
    {
      title: '扩展',
      desc: '只有当第一件事跑通以后，再接更多工具或更自动化的流程。'
    }
  ],
  faqItems: [
    {
      question: '现在的 DoWhiz 到底是什么？',
      answer:
        '现在 DoWhiz 先从 Oliver 开始：一个会在你的工具里工作、把结果带回来、并且可以逐步建立信任的 AI operator。'
    },
    {
      question: '我应该怎么开始？',
      answer:
        '先连一个工具，交给 Oliver 一个真实任务，复核结果，然后保存让它更好用的偏好。'
    },
    {
      question: 'Oliver 会记住上下文吗？',
      answer:
        '会。Oliver 可以保存你的偏好、历史任务和后续跟进所需的共享记忆，这样下次不用从头再讲。'
    },
    {
      question: '我需要把个人账号密码交给 DoWhiz 吗？',
      answer:
        '不需要。DoWhiz 更强调受控访问和可审阅执行，而不是让你交出宽泛的个人凭证。'
    },
    {
      question: 'Oliver 能同时处理个人和工作任务吗？',
      answer:
        '可以，只要任务本身清楚、可审阅、适合被辅助执行。研究、写作、材料整理、跟进和工具内执行都很适合。'
    },
    {
      question: 'DoWhiz 只做个人场景吗？',
      answer:
        '产品先从个人可用性出发。当 Oliver 已经在一个人的工作流里被证明有用，再自然扩展到 SMB 的共享流程。'
    }
  ],
  blogPosts: [
    {
      tag: 'Workflow guide',
      title: 'AI 工作流检查清单',
      date: '2026 年 3 月 27 日',
      excerpt: '关于触发器、质量门槛和可靠交付的一份实操清单。',
      link: '/blog/ai-workflow-automation-checklist/'
    },
    {
      tag: 'Engineering',
      title: 'GitHub issue 自动化最佳实践',
      date: '2026 年 2 月 26 日',
      excerpt: '让 issue 到 PR 的推进更稳、更容易交付的一套做法。',
      link: '/blog/github-issue-automation-best-practices/'
    },
    {
      tag: 'Email',
      title: '邮件自动化操作手册',
      date: '2026 年 2 月 26 日',
      excerpt: '如何把收件箱里的请求转成结构化执行和明确交付。',
      link: '/blog/email-task-automation-playbook/'
    },
    {
      tag: 'Memory',
      title: 'AI agent memory 最佳实践',
      date: '2026 年 2 月 26 日',
      excerpt: '哪些该记住，哪些该清掉，以及怎样让跨渠道跟进真的更好用。',
      link: '/blog/ai-agent-memory-best-practices/'
    }
  ],
  labels: {
    blogEyebrow: '博客',
    blogTitle: '关于 operator-style automation 的一些记录',
    blogIntro: '围绕记忆、收件箱、自动化质量和真实世界里 AI operator 的边界做的实践总结。',
    blogHeaderButton: '查看全部',
    blogLinkLabel: '阅读全文',
    accessPlaybookTitle: 'Oliver 的扩展方式',
    accessPlaybookDescription:
      '不需要一口气做完所有设置。先把一个场景跑通，再增加下一个工具或下一段重复流程。',
    accessPlaybookLink: '查看 Trust & Safety',
    faqCta: '打开完整帮助中心',
    footerTitle: '更多内容',
    footerTagline: 'Oliver 把请求变成可审阅、可复用、能落地的结果。',
    footerPill: '先服务个人，再扩展到共享流程。',
    footerBottomSecondary: '为清楚的 onboarding、受控执行和真实工作而设计。'
  },
  footerLinks: [
    { href: '/privacy/', label: '隐私政策' },
    { href: '/terms/', label: '服务条款' },
    { href: '/trust-safety/', label: 'Trust & Safety' },
    { href: '/integrations/', label: '集成' },
    { href: '/user-guide/', label: '使用指南' },
    { href: 'https://www.dowhiz.com/help-center/', label: '帮助中心' },
    { href: '/solutions/ai-workflow-automation/', label: 'AI 工作流自动化' },
    { href: '/solutions/github-issue-automation/', label: 'GitHub Issue 自动化' },
    { href: '/solutions/slack-task-automation/', label: 'Slack 自动化' },
    { href: '/solutions/email-task-automation/', label: '邮件自动化' },
    { href: '/solutions/google-docs-automation/', label: 'Google Docs 自动化' }
  ]
};

export function getLandingContent(locale = 'en-US') {
  return locale === 'zh-CN' ? ZH_LANDING_CONTENT : EN_LANDING_CONTENT;
}
