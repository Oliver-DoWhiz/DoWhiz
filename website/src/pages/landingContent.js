const EN_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz | Oliver works in your tools',
    description:
      'Use Oliver from the landing page without starting in login. Email Oliver right away, then add Slack, Discord, GitHub, Notion, or Lark when you want more connected workflows.',
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
      'Email Oliver right away. Then add Slack, Discord, GitHub, Notion, or Lark when you want Oliver inside those workflows.',
    caption:
      'Start with the request itself. Oliver can guide account linking, saved memory, and app setup later, inside the conversation.',
    primaryCta: 'Email Oliver now',
    secondaryCta: 'See real examples',
    manageAnonymous: 'Manage setup later',
    manageAuthenticated: 'Open your setup',
    contactSubject: 'A first task for Oliver',
    contactBody:
      'Hi Oliver,\n\nHere is the first task I want help with:\n\n- Context:\n- What done looks like:\n- Any deadline:\n\nThanks!',
    toolsEyebrow: 'Where Oliver works today',
    toolsHintAnonymous:
      'Email is the only true no-login start today. The other channels open setup when you are ready to connect them.',
    toolsHintAuthenticated: 'If you are signed in, these buttons start the real connect flow.',
    toolsFootnote:
      'Use setup when you want saved memory, linked identities, or more app connections. It does not need to be your first step.',
    directEyebrow: 'Fastest first move',
    directTitle: 'Send the first request by email.',
    directDescription:
      'You do not need an account for the first message. Oliver can reply first and guide linking only if it helps.',
    directBadge: 'No login needed for the first email',
    directSubnote: 'Best for a first request, a follow-up, or any draft-heavy task.',
    directActionLabel: 'Compose email',
    actionLabels: {
      connect: 'Connect',
      setup: 'Open setup',
      loading: 'Opening...'
    },
    tools: [
      {
        key: 'slack',
        label: 'Slack',
        availability: 'Connect when ready',
        description: 'Bring Oliver into the team chat where requests already show up.',
        monogram: 'S',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        availability: 'Connect when ready',
        description: 'Add Oliver to fast back-and-forth coordination and community work.',
        monogram: 'D',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        availability: 'Connect for repo work',
        description: 'Connect issues, follow-up, and code-adjacent requests.',
        monogram: 'GH',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        availability: 'Connect for docs',
        description: 'Bring Oliver into notes, docs, and task context when it matters.',
        monogram: 'N',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        availability: 'Connect for ops',
        description: 'Use Oliver in coordination-heavy operations and follow-through.',
        monogram: 'L',
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
        'Email is the fastest real starting point today. You can send Oliver a request immediately, without creating an account first.'
    },
    {
      question: 'Do I need an account before the first request?',
      answer:
        'Not for the first email. Oliver can begin helping first, then guide account linking or setup later if it actually helps the workflow.'
    },
    {
      question: 'Can I use Slack or Discord immediately?',
      answer:
        'Today, email is the cleanest no-login entry point. Slack, Discord, GitHub, Notion, and Lark are best added when you are ready to connect them through setup.'
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
    faqIntro: 'Short answers about starting in email, connecting more channels later, and staying in control.',
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
      '认识 Oliver。你不需要先登录再开始：现在就可以先给 Oliver 发邮件，等真正需要时再连接 Slack、Discord、GitHub、Notion 或 Lark。',
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
      '你现在就可以先通过 Email 使用 Oliver。等你想让它进入 Slack、Discord、GitHub、Notion 或 Lark 的工作流时，再去连接这些渠道。',
    caption:
      '先从请求本身开始。真正需要的时候，Oliver 可以在对话里继续引导账号绑定、memory 和 app setup。',
    primaryCta: '现在给 Oliver 发邮件',
    secondaryCta: '看真实例子',
    manageAnonymous: '稍后再管理 setup',
    manageAuthenticated: '打开你的 setup',
    contactSubject: '给 Oliver 的第一个任务',
    contactBody:
      '你好 Oliver，\n\n这是我想先让你帮忙处理的第一个任务：\n\n- 背景：\n- 什么算完成：\n- 截止时间：\n\n谢谢！',
    toolsEyebrow: 'Oliver 现在可工作的渠道',
    toolsHintAnonymous:
      '今天真正可以无登录直接开始的入口是 Email。其他渠道会在你准备好连接时带你进入 setup。',
    toolsHintAuthenticated: '如果你已经登录，这些按钮会直接启动真实连接流程。',
    toolsFootnote:
      '只有当你希望 Oliver 记住更多上下文、绑定身份、或接入更多 app 时，才需要进入 setup。',
    directEyebrow: '最快的第一步',
    directTitle: '先直接发第一封邮件。',
    directDescription:
      '第一条消息不需要先注册账号。Oliver 可以先接住请求，再在真正有帮助的时候引导你完成绑定。',
    directBadge: '第一封邮件不需要登录',
    directSubnote: '尤其适合第一次尝试、邮件往来、或需要起草的任务。',
    directActionLabel: '写邮件给 Oliver',
    actionLabels: {
      connect: '连接',
      setup: '打开 setup',
      loading: '打开中...'
    },
    tools: [
      {
        key: 'slack',
        label: 'Slack',
        availability: '准备好时再连接',
        description: '当任务已经发生在团队聊天里时，把 Oliver 带进去。',
        monogram: 'S',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        availability: '准备好时再连接',
        description: '把 Oliver 放进高频来回沟通和社区协作里。',
        monogram: 'D',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        availability: '用于 repo 工作',
        description: '把 issue、跟进和代码周边请求交给 Oliver 处理。',
        monogram: 'GH',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        availability: '用于文档上下文',
        description: '当文档、笔记和任务上下文很关键时，再把 Oliver 接进去。',
        monogram: 'N',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        availability: '用于协作运营',
        description: '让 Oliver 进入运营协作和需要持续跟进的沟通里。',
        monogram: 'L',
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
        '今天最快、最真实的入口是 Email。你现在就可以先给 Oliver 发一封邮件，不需要先注册账号。'
    },
    {
      question: '第一条消息之前必须先有账号吗？',
      answer:
        '第一封邮件不需要。Oliver 可以先开始帮忙，等真的有帮助时，再在对话里引导你完成账号绑定或更多 setup。'
    },
    {
      question: 'Slack 或 Discord 现在能直接开始吗？',
      answer:
        '目前最干净的无登录入口是 Email。Slack、Discord、GitHub、Notion 和 Lark 更适合在你准备好连接这些渠道时再接入。'
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
    faqIntro: '只回答最关键的问题：怎么开始、什么时候再连接更多渠道、以及你是否还掌控流程。',
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
