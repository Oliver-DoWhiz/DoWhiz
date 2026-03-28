const EN_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz | Oliver works in your tools',
    description:
      'Start with Oliver in email, Slack, or Discord without logging in first. GitHub, Notion, and Lark work can start with a real request now and connect later when you want saved workflows.',
    canonicalUrl: 'https://dowhiz.com/',
    ogLocale: 'en_US',
    themeColor: '#2C2C2E',
    htmlLang: 'en'
  },
  nav: {
    homePath: '/',
    links: [
      { href: '#channels', label: 'Start now' },
      { href: '#examples', label: 'What Oliver can do' },
      { href: '#faq', label: 'FAQ' }
    ],
    signIn: 'Manage setup',
    dashboard: 'Your Oliver',
    signOut: 'Sign out',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: 'Email Oliver'
  },
  hero: {
    eyebrow: 'Start with a real request',
    title: 'Use Oliver where you already work',
    subtitle:
      'Start in email, Slack, or Discord right away. GitHub, Notion, and Lark can start with a real request and connect later if it helps',
    primaryCta: 'Get DoWhiz free',
    secondaryCta: 'See example',
    manageAnonymous: 'Manage setup',
    manageAuthenticated: 'Open your setup',
    contactSubject: 'A first task for Oliver',
    contactBody:
      'Hi Oliver,\n\nHere is the first task I want help with:\n\n- Context:\n- What done looks like:\n- Any deadline:\n\nThanks!',
    toolsEyebrow: 'Choose a channel',
    toolsHintAnonymous:
      'Hover to pause. Click a channel to start. GitHub, Notion, and Lark can begin from email first.',
    toolsHintAuthenticated: 'Hover to pause. Signed-in channels can open direct connect flows when needed.',
    toolsFootnote:
      'Start in a channel now. Use setup only when you want saved memory or deeper app access.',
    autoplayLabel: 'Autoplay',
    pausedLabel: 'Paused',
    entryEyebrow: 'Fastest start',
    previewEyebrow: 'What it looks like',
    youLabel: 'You',
    actionLabels: {
      connect: 'Connect',
      loading: 'Opening...'
    },
    tools: [
      {
        key: 'email',
        label: 'Email',
        anonymousStatus: 'Direct compose',
        authenticatedStatus: 'Direct compose',
        anonymousActionLabel: 'Send request',
        authenticatedActionLabel: 'Send request',
        description: 'Send the request you already have in mind',
        samplePrompt: 'Turn these notes into a concise follow-up email',
        sampleReply: 'I can draft the reply, tighten the tone, and point out what is still missing',
        accent: '#ff8a3d'
      },
      {
        key: 'slack',
        label: 'Slack',
        anonymousStatus: 'Public install',
        authenticatedStatus: 'Connect workspace',
        anonymousActionLabel: 'Add bot',
        authenticatedActionLabel: 'Connect',
        description: 'Install Oliver and ask in Slack',
        samplePrompt: 'Summarize this channel and list the next three actions',
        sampleReply: 'I can pull the thread context and return a clean action list',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        anonymousStatus: 'Public invite',
        authenticatedStatus: 'Connect server',
        anonymousActionLabel: 'Add bot',
        authenticatedActionLabel: 'Connect',
        description: 'Invite Oliver and start in your server',
        samplePrompt: 'Read this discussion and turn it into a plan for the week',
        sampleReply: 'I can organize the thread into owners, deadlines, and a reply-ready summary',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect repo access',
        anonymousActionLabel: 'Send request',
        authenticatedActionLabel: 'Connect',
        description: 'Start with repo context, connect later if needed',
        samplePrompt: 'Review these issues and tell me what should be fixed first',
        sampleReply: 'I can rank the work, flag blockers, and draft the next follow-up',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect docs access',
        anonymousActionLabel: 'Send request',
        authenticatedActionLabel: 'Connect',
        description: 'Start with docs or notes work, connect later if helpful',
        samplePrompt: 'Turn these notes into a clean study outline',
        sampleReply: 'I can organize the draft, tighten the structure, and flag what still needs research',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect workspace',
        anonymousActionLabel: 'Send request',
        authenticatedActionLabel: 'Connect',
        description: 'Start ops follow-up now, connect later if needed',
        samplePrompt: 'Draft a follow-up plan from this meeting summary',
        sampleReply: 'I can turn it into owners, deadlines, and an update you can send',
        accent: '#3f88ff'
      }
    ]
  },
  demo: {
    eyebrow: 'Real product examples',
    title: 'Watch the request-to-result loop before you change your workflow.',
    intro: 'One longer walkthrough and three faster mobile examples.',
    desktopTitle: 'Desktop walkthrough',
    desktopDescription: 'See the request-to-result loop in one flow.',
    desktopVideoId: 'IsSOTSYIIIY',
    desktopVideoHref: 'https://youtu.be/IsSOTSYIIIY',
    desktopCta: 'Open on YouTube',
    shortsTitle: 'Mobile quick looks',
    shortsDescription: 'Short examples built for a faster scan.',
    shorts: [
      {
        title: 'Mobile demo 01',
        videoId: 'SI9mxW_Top0',
        href: 'https://www.youtube.com/shorts/SI9mxW_Top0'
      },
      {
        title: 'Mobile demo 02',
        videoId: 'PSsJ7WBk71w',
        href: 'https://www.youtube.com/shorts/PSsJ7WBk71w'
      },
      {
        title: 'Mobile demo 03',
        videoId: '5H9g3LOGkMc',
        href: 'https://www.youtube.com/shorts/5H9g3LOGkMc'
      }
    ]
  },
  examples: {
    eyebrow: 'What you can ask',
    title: 'Start with something concrete, not a toy prompt.',
    intro: 'Good first requests are real, bounded, and worth handing off for a first pass.',
    cards: [
      {
        tag: 'Research',
        title: 'Deep research briefs',
        description: 'Compare options and return a concise decision memo.'
      },
      {
        tag: 'Inbox',
        title: 'Email triage and drafts',
        description: 'Draft replies and keep follow-ups moving.'
      },
      {
        tag: 'Writing',
        title: 'Study and drafting support',
        description: 'Outline, revise, and tighten documents.'
      },
      {
        tag: 'Tax prep',
        title: 'Document organization',
        description: 'Sort filing materials and flag what is missing.'
      },
      {
        tag: 'Content',
        title: 'Posts with approval',
        description: 'Draft and schedule posts for review.'
      },
      {
        tag: 'GitHub',
        title: 'Repo follow-up',
        description: 'Summarize issues and keep code-adjacent work moving.'
      }
    ]
  },
  control: {
    eyebrow: 'Control and trust',
    title: 'Oliver can help first while you stay in control.',
    points: [
      'Start in the channel that already fits the request',
      'Keep sensitive work reviewable before anything final is sent',
      'Save setup and memory only when you want Oliver to keep context'
    ]
  },
  faqItems: [
    {
      question: 'What is the fastest way to start?',
      answer:
        'Email is still the fastest no-login start. Slack and Discord also have direct add flows, and GitHub, Notion, or Lark work can start with a first request before you connect anything.'
    },
    {
      question: 'Do I need an account before the first request?',
      answer:
        'No. You can start with email, Slack, or Discord without creating an account first. GitHub, Notion, and Lark work can start with a request first and be linked later when deeper access helps.'
    },
    {
      question: 'Can I use Slack or Discord immediately?',
      answer:
        'Yes. Slack and Discord open public install or invite flows today. GitHub, Notion, and Lark still start with a request first, then connect later when you want saved app access.'
    },
    {
      question: 'What kinds of first requests work best?',
      answer:
        'Research, drafting, inbox follow-up, document organization, repo-side coordination, and posts that still wait for your approval are all good first uses.'
    },
    {
      question: 'Do I stay in control?',
      answer:
        'Yes. You decide which apps Oliver can use, when setup is worth saving, and which sensitive actions should stay reviewable.'
    }
  ],
  labels: {
    faqEyebrow: 'Questions',
    faqTitle: 'The essentials before you start',
    faqIntro: 'Short answers about starting without login, choosing a real channel first, and deciding later which connections are worth saving.',
    faqLinkLabel: 'Open the Help Center',
    footerTitle: 'Resources',
    footerTagline: 'Oliver helps turn requests into finished work.',
    footerPill: 'Start with the request. Expand later.',
    footerBottomSecondary: 'Use first. Connect more only when it helps.'
  },
  footerLinks: [
    { href: '/privacy/', label: 'Privacy' },
    { href: '/terms/', label: 'Terms of Service' },
    { href: '/trust-safety/', label: 'Trust & Safety' },
    { href: '/integrations/', label: 'Integrations' },
    { href: 'https://www.dowhiz.com/help-center/', label: 'Help Center' },
    { href: '/user-guide/', label: 'User Guide' }
  ]
};

const ZH_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz 中文 | Oliver 在你的工具里工作',
    description:
      '认识 Oliver。你不需要先登录再开始：现在就可以直接通过 Email、Slack 或 Discord 开始使用。GitHub、Notion 和 Lark 相关任务也可以先发出真实请求，之后再决定是否连接。',
    canonicalUrl: 'https://dowhiz.com/cn',
    ogLocale: 'zh_CN',
    themeColor: '#2C2C2E',
    htmlLang: 'zh-CN'
  },
  nav: {
    homePath: '/cn',
    links: [
      { href: '#channels', label: '现在开始' },
      { href: '#examples', label: 'Oliver 能做什么' },
      { href: '#faq', label: '常见问题' }
    ],
    signIn: '管理 setup',
    dashboard: '你的 Oliver',
    signOut: '退出登录',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: '给 Oliver 发邮件'
  },
  hero: {
    eyebrow: '先从一个真实请求开始',
    title: '在你常用的渠道里用 Oliver',
    subtitle:
      '现在就可以从 Email、Slack 或 Discord 开始，GitHub、Notion 和 Lark 相关工作也可以先从一个真实请求开始，之后再按需连接',
    primaryCta: '免费开始用 DoWhiz',
    secondaryCta: '看示例',
    manageAnonymous: '管理 setup',
    manageAuthenticated: '打开你的 setup',
    contactSubject: '给 Oliver 的第一个任务',
    contactBody:
      '你好 Oliver，\n\n这是我想先让你帮忙处理的第一个任务：\n\n- 背景：\n- 什么算完成：\n- 截止时间：\n\n谢谢！',
    toolsEyebrow: '选择一个渠道',
    toolsHintAnonymous:
      '悬停会暂停轮播，点击就直接开始。GitHub、Notion 和 Lark 可以先从邮件请求开始。',
    toolsHintAuthenticated: '悬停会暂停轮播。登录后，这些入口也可以在需要时直接启动连接流程。',
    toolsFootnote:
      '现在就先在一个渠道里开始。只有当你想保存 memory 或接入更深权限时，再进入 setup。',
    autoplayLabel: '自动播放',
    pausedLabel: '已暂停',
    entryEyebrow: '最快开始方式',
    previewEyebrow: '交互会是什么样',
    youLabel: '你',
    actionLabels: {
      connect: '连接',
      loading: '打开中...'
    },
    tools: [
      {
        key: 'email',
        label: 'Email',
        anonymousStatus: '直接写邮件',
        authenticatedStatus: '直接写邮件',
        anonymousActionLabel: '发送请求',
        authenticatedActionLabel: '发送请求',
        description: '把你已经想好的请求直接发出去',
        samplePrompt: '把这些要点整理成一封简洁的跟进邮件',
        sampleReply: '我可以先起草回复、收紧语气，并指出还缺什么信息',
        accent: '#ff8a3d'
      },
      {
        key: 'slack',
        label: 'Slack',
        anonymousStatus: '公开安装',
        authenticatedStatus: '连接工作区',
        anonymousActionLabel: '添加机器人',
        authenticatedActionLabel: '连接',
        description: '装好 Oliver 后直接在 Slack 里开始',
        samplePrompt: '帮我总结这个频道，并列出接下来三件事',
        sampleReply: '我可以把线程上下文整理成清晰的行动清单',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        anonymousStatus: '公开邀请',
        authenticatedStatus: '连接服务器',
        anonymousActionLabel: '添加机器人',
        authenticatedActionLabel: '连接',
        description: '邀请 Oliver 后直接在 server 里开始',
        samplePrompt: '读一下这段讨论，并把它整理成本周执行计划',
        sampleReply: '我可以把讨论整理成负责人、截止时间和可直接发送的总结',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接仓库权限',
        anonymousActionLabel: '发送请求',
        authenticatedActionLabel: '连接',
        description: '先带着 repo 上下文开始，需要时再连接',
        samplePrompt: '看看这些 issues，告诉我应该先修哪几个',
        sampleReply: '我可以帮你排优先级、指出 blocker，并起草下一步跟进',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接文档权限',
        anonymousActionLabel: '发送请求',
        authenticatedActionLabel: '连接',
        description: '先从文档或笔记任务开始，有需要再连接',
        samplePrompt: '把这些笔记整理成一份清楚的学习提纲',
        sampleReply: '我可以先整理结构、收紧逻辑，并指出还要补哪些研究',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接工作区',
        anonymousActionLabel: '发送请求',
        authenticatedActionLabel: '连接',
        description: '先从协作跟进任务开始，需要时再连接',
        samplePrompt: '根据这次会议总结，起草一个后续推进计划',
        sampleReply: '我可以把它整理成负责人、时间点和一段可直接发送的更新',
        accent: '#3f88ff'
      }
    ]
  },
  demo: {
    eyebrow: '真实产品演示',
    title: '先看 Oliver 如何把请求变成结果，而不是先看一整套流程说明。',
    intro: '一个较完整的桌面演示，加上三个更短的移动端例子。',
    desktopTitle: '桌面完整演示',
    desktopDescription: '一条龙看完从请求到结果的过程。',
    desktopVideoId: 'IsSOTSYIIIY',
    desktopVideoHref: 'https://youtu.be/IsSOTSYIIIY',
    desktopCta: '去 YouTube 看',
    shortsTitle: '移动端快速演示',
    shortsDescription: '更短、更适合快速浏览的几个例子。',
    shorts: [
      {
        title: '移动端演示 01',
        videoId: 'SI9mxW_Top0',
        href: 'https://www.youtube.com/shorts/SI9mxW_Top0'
      },
      {
        title: '移动端演示 02',
        videoId: 'PSsJ7WBk71w',
        href: 'https://www.youtube.com/shorts/PSsJ7WBk71w'
      },
      {
        title: '移动端演示 03',
        videoId: '5H9g3LOGkMc',
        href: 'https://www.youtube.com/shorts/5H9g3LOGkMc'
      }
    ]
  },
  examples: {
    eyebrow: '你可以怎么问',
    title: '第一条消息最好是真实任务，而不是试玩 prompt。',
    intro: '最好的第一次请求，通常都具体、可复核，而且值得先让 Oliver 做一版。',
    cards: [
      {
        tag: 'Research',
        title: 'Deep research 简报',
        description: '比较方案，最后给你一份能直接判断的结果。'
      },
      {
        tag: 'Inbox',
        title: '邮件整理和回复起草',
        description: '起草回复，同时把后续动作继续往前推。'
      },
      {
        tag: 'Writing',
        title: '学习支持和文稿起草',
        description: '做提纲、改写、收紧结构。'
      },
      {
        tag: 'Tax prep',
        title: '材料整理',
        description: '整理报税前的文件并指出还缺什么。'
      },
      {
        tag: 'Content',
        title: '发帖前的草稿与排期',
        description: '先起草和排期，最终仍由你确认。'
      },
      {
        tag: 'GitHub',
        title: 'Repo 跟进',
        description: '整理 issue 上下文，推进和代码相关的后续工作。'
      }
    ]
  },
  control: {
    eyebrow: '控制与信任',
    title: 'Oliver 可以先开始帮忙，而你仍然掌控节奏。',
    points: [
      '先在最适合任务的那个渠道里开始',
      '敏感动作仍然可以保持可审阅、可确认',
      '只有当你希望保留上下文时，再保存 setup 和 memory'
    ]
  },
  faqItems: [
    {
      question: '最快怎么开始？',
      answer:
        'Email 仍然是最快的无登录入口。Slack 和 Discord 也已经有直接可用的公开安装入口，而 GitHub、Notion、Lark 相关任务也可以先发出第一个请求，再决定要不要连接。'
    },
    {
      question: '第一条消息之前必须先有账号吗？',
      answer:
        '不需要。你可以先通过 Email、Slack 或 Discord 开始，不用先注册账号。GitHub、Notion 和 Lark 相关任务也可以先发请求，之后再在更深权限真的有帮助时完成连接。'
    },
    {
      question: 'Slack 或 Discord 现在能直接开始吗？',
      answer:
        '可以。Slack 和 Discord 现在就会打开公开安装或邀请入口。GitHub、Notion 和 Lark 目前仍然更适合先发出请求，再在你希望保存 app 访问权限时完成连接。'
    },
    {
      question: '第一条消息最适合发什么？',
      answer:
        '研究、起草、收件箱跟进、材料整理、repo 协调，以及需要你最终确认的内容发布，都是很好的开始。'
    },
    {
      question: '我还掌控流程吗？',
      answer:
        '掌控权仍然在你手里。你决定连哪些工具、哪些动作要先审阅，以及 Oliver 什么时候值得保存更多上下文。'
    }
  ],
  labels: {
    faqEyebrow: '常见问题',
    faqTitle: '开始之前真正需要知道的几件事',
    faqIntro: '只回答最关键的问题：如何无登录开始、先用哪个真实渠道，以及哪些连接值得以后再保存。',
    faqLinkLabel: '打开帮助中心',
    footerTitle: '资源',
    footerTagline: 'Oliver 帮你把请求变成结果。',
    footerPill: '先从请求开始，再决定是否扩展。',
    footerBottomSecondary: '先用起来，再在真正有帮助的时候连接更多。'
  },
  footerLinks: [
    { href: '/privacy/', label: '隐私政策' },
    { href: '/terms/', label: '服务条款' },
    { href: '/trust-safety/', label: 'Trust & Safety' },
    { href: '/integrations/', label: '集成' },
    { href: 'https://www.dowhiz.com/help-center/', label: '帮助中心' },
    { href: '/user-guide/', label: '使用指南' }
  ]
};

export function getLandingContent(locale = 'en-US') {
  return locale === 'zh-CN' ? ZH_LANDING_CONTENT : EN_LANDING_CONTENT;
}
