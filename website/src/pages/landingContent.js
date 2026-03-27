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
    title: 'Use Oliver in a channel you already have open.',
    subtitle:
      'Start in email, Slack, or Discord right away. For GitHub, Notion, or Lark work, send the first request now and connect the app later only if it helps.',
    caption:
      'Get first value before setup. Oliver can guide linking, saved memory, and deeper app access inside the conversation when needed.',
    primaryCta: 'Email Oliver now',
    secondaryCta: 'See real examples',
    manageAnonymous: 'Manage setup later',
    manageAuthenticated: 'Open your setup',
    contactSubject: 'A first task for Oliver',
    contactBody:
      'Hi Oliver,\n\nHere is the first task I want help with:\n\n- Context:\n- What done looks like:\n- Any deadline:\n\nThanks!',
    toolsEyebrow: 'Where Oliver works today',
    toolsHintAnonymous:
      'Slack and Discord open public add flows. GitHub, Notion, and Lark start with a real request first, then connect later if you want saved access.',
    toolsHintAuthenticated: 'If you are signed in, these buttons start the real connect flow.',
    toolsFootnote:
      'Sign in only when you want saved memory, linked identities, or deeper app access. It does not need to be your first step.',
    directEyebrow: 'Fastest first move',
    directTitle: 'Email is still the simplest first move.',
    directDescription:
      'If you want the lightest no-login start, send the first request by email and let Oliver guide any later setup in context.',
    directBadge: 'Fastest no-login path',
    directSubnote: 'Best for a brand-new request, a follow-up, or any drafting-heavy task.',
    directActionLabel: 'Compose email',
    actionLabels: {
      connect: 'Connect',
      loading: 'Opening...'
    },
    tools: [
      {
        key: 'slack',
        label: 'Slack',
        anonymousStatus: 'Public install',
        authenticatedStatus: 'Connect workspace',
        anonymousActionLabel: 'Add bot',
        description: 'Add Oliver to a workspace in one click, then ask the first thing you need handled in Slack.',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        anonymousStatus: 'Public invite',
        authenticatedStatus: 'Connect server',
        anonymousActionLabel: 'Add bot',
        description: 'Invite Oliver into your server and start with a real request there.',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect repo access',
        anonymousActionLabel: 'Send request',
        description: 'Send repo context right away, then connect GitHub later for saved repo-side follow-up.',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect docs access',
        anonymousActionLabel: 'Send request',
        description: 'Start with a docs or notes request now, then connect Notion when you want workspace context.',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        anonymousStatus: 'Start with email',
        authenticatedStatus: 'Connect workspace',
        anonymousActionLabel: 'Send request',
        description: 'Start ops and coordination work now, then connect Lark when you want persistent access.',
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
    title: '在你已经打开的渠道里直接开始用 Oliver。',
    subtitle:
      '你现在就可以直接通过 Email、Slack 或 Discord 开始。对于 GitHub、Notion 或 Lark 相关任务，也可以先发出第一个请求，只有在真正有帮助时再连接 app。',
    caption:
      '先拿到第一轮价值，再决定要不要 setup。真正需要的时候，Oliver 可以在对话里继续引导账号绑定、memory 和更深的 app 接入。',
    primaryCta: '现在给 Oliver 发邮件',
    secondaryCta: '看真实例子',
    manageAnonymous: '稍后再管理 setup',
    manageAuthenticated: '打开你的 setup',
    contactSubject: '给 Oliver 的第一个任务',
    contactBody:
      '你好 Oliver，\n\n这是我想先让你帮忙处理的第一个任务：\n\n- 背景：\n- 什么算完成：\n- 截止时间：\n\n谢谢！',
    toolsEyebrow: 'Oliver 现在可工作的渠道',
    toolsHintAnonymous:
      'Slack 和 Discord 会直接打开公开安装入口。GitHub、Notion 和 Lark 则可以先发出真实请求，之后如果你希望保存访问权限，再去连接。',
    toolsHintAuthenticated: '如果你已经登录，这些按钮会直接启动真实连接流程。',
    toolsFootnote:
      '只有当你希望 Oliver 记住更多上下文、绑定身份、或接入更深的 app 权限时，才需要登录进入 setup。',
    directEyebrow: '最快的第一步',
    directTitle: 'Email 仍然是最轻的第一步。',
    directDescription:
      '如果你想用最轻的无登录方式开始，先直接发邮件就可以。后续真的有帮助时，再让 Oliver 在对话里引导 setup。',
    directBadge: '最快的无登录入口',
    directSubnote: '尤其适合第一次尝试、后续追问，或任何需要起草的任务。',
    directActionLabel: '写邮件给 Oliver',
    actionLabels: {
      connect: '连接',
      loading: '打开中...'
    },
    tools: [
      {
        key: 'slack',
        label: 'Slack',
        anonymousStatus: '公开安装',
        authenticatedStatus: '连接工作区',
        anonymousActionLabel: '添加机器人',
        description: '一键把 Oliver 加进 Slack workspace，然后直接在里面发出第一个请求。',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        anonymousStatus: '公开邀请',
        authenticatedStatus: '连接服务器',
        anonymousActionLabel: '添加机器人',
        description: '把 Oliver 邀请进你的 server，然后直接在那里开始真实任务。',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接仓库权限',
        anonymousActionLabel: '发送请求',
        description: '先把 repo 上下文发给 Oliver，之后如果你希望它持续跟进，再连接 GitHub。',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接文档权限',
        anonymousActionLabel: '发送请求',
        description: '先从文档或笔记相关请求开始，等你希望 Oliver 进入 workspace 上下文时再连接 Notion。',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        anonymousStatus: '先发邮件开始',
        authenticatedStatus: '连接工作区',
        anonymousActionLabel: '发送请求',
        description: '先从协作运营或跟进型任务开始，之后如果需要持续权限，再连接 Lark。',
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
