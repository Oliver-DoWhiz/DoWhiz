const EN_LANDING_CONTENT = {
  metadata: {
    title: 'DoWhiz | Oliver works in your tools',
    description:
      'Meet Oliver, the trusted AI operator for work and life. Start from the homepage, connect one app, send one real task, and review the result.',
    canonicalUrl: 'https://dowhiz.com/',
    ogLocale: 'en_US',
    themeColor: '#2C2C2E',
    htmlLang: 'en'
  },
  nav: {
    homePath: '/',
    links: [
      { href: '#examples', label: 'Examples' },
      { href: '#how-it-starts', label: 'How it starts' },
      { href: '#faq', label: 'FAQ' }
    ],
    signIn: 'Sign in',
    dashboard: 'Oliver setup',
    signOut: 'Sign out',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: 'Email Oliver'
  },
  hero: {
    eyebrow: 'Meet Oliver',
    title: 'Tell Oliver what needs doing.',
    subtitle:
      'Oliver is the trusted AI operator that works in email, Slack, Discord, GitHub, Notion, and Lark, then brings back finished work you can review.',
    caption: 'Start with one app. Keep the first task small.',
    primaryCtaAnonymous: 'Start with one app',
    primaryCtaAuthenticated: 'Continue onboarding',
    contactSubject: 'A first task for Oliver',
    contactBody:
      'Hi Oliver,\n\nHere is the first task I want help with:\n\n- Context:\n- What done looks like:\n- Any deadline:\n\nThanks!',
    toolsEyebrow: 'Choose a real starting point',
    toolsHintAnonymous: 'Email works right away. The other tools take you into setup.',
    toolsHintAuthenticated: 'If you are signed in, these buttons start the real connect flow.',
    actionLabels: {
      email: 'Send now',
      connect: 'Connect',
      setup: 'Open setup',
      loading: 'Opening...'
    },
    tools: [
      {
        key: 'email',
        label: 'Email',
        description: 'Send Oliver a request right away.',
        monogram: '@',
        accent: '#ff8a3d'
      },
      {
        key: 'slack',
        label: 'Slack',
        description: 'Connect the place where chat work lands.',
        monogram: 'S',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        description: 'Start where fast back-and-forth already happens.',
        monogram: 'D',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        description: 'Connect repo work, issues, and follow-up.',
        monogram: 'GH',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        description: 'Connect docs, notes, and task context.',
        monogram: 'N',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        description: 'Connect operations and coordination.',
        monogram: 'L',
        accent: '#3f88ff'
      }
    ],
    flowEyebrow: 'How onboarding starts',
    flowSteps: [
      'Connect one app',
      'Give Oliver one task',
      'Review the result',
      'Repeat what works'
    ],
    previewEyebrow: 'A good first task',
    previewRequest:
      'Draft a reply to this email, pull the action items into Notion, and hold anything sensitive for review.',
    previewResultTitle: 'What comes back',
    previewResults: ['Reply draft ready', 'Tasks captured', 'Approval items flagged']
  },
  demo: {
    eyebrow: 'Real product examples',
    title: 'See Oliver working, not just described.',
    intro: 'One desktop walkthrough and three quick mobile demos.',
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
  story: {
    eyebrow: 'How it starts',
    title: 'The first win should be small, clear, and useful.',
    intro: 'You are not setting up a whole new system. You are proving one real loop.',
    steps: [
      {
        id: '01',
        title: 'Connect one app',
        description: 'Choose the place where work already lands.'
      },
      {
        id: '02',
        title: 'Give one real task',
        description: 'Pick something concrete, reviewable, and worth handing off.'
      },
      {
        id: '03',
        title: 'Review and repeat',
        description: 'Check the output, save the memory, and only then expand.'
      }
    ],
    controlEyebrow: 'Review and control',
    controlTitle: 'Oliver works with your approval, not in the dark.',
    controlPoints: [
      'Connect only the apps you want Oliver to use',
      'Review drafts before sensitive actions',
      'Keep your preferences and memory in one place'
    ]
  },
  examples: {
    eyebrow: 'Useful from day one',
    title: 'Good early tasks span both work and life.',
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
  faqItems: [
    {
      question: 'What is DoWhiz now?',
      answer:
        'DoWhiz starts with Oliver: a trusted AI operator who works in your tools, returns finished work, and helps you build trust one task at a time.'
    },
    {
      question: 'What should I do first?',
      answer:
        'Connect one app, give Oliver one real task, review the result, and save the preference or memory that made it useful.'
    },
    {
      question: 'What kinds of work fit best?',
      answer:
        'Good first uses include research, drafting, document organization, inbox follow-up, repo-side coordination, and posts that still wait for your approval.'
    },
    {
      question: 'Do I stay in control?',
      answer:
        'Yes. You choose what gets connected, you can review sensitive work before it is sent, and you can keep setup narrow while Oliver earns trust.'
    },
    {
      question: 'Is DoWhiz only for personal use?',
      answer:
        'The product starts with personal usefulness first. Once Oliver is trusted in one workflow, the same setup can grow into shared SMB routines later.'
    }
  ],
  labels: {
    faqEyebrow: 'Questions',
    faqTitle: 'The essentials before you start',
    faqIntro: 'Short answers about trust, fit, and what onboarding really looks like.',
    faqLinkLabel: 'Open the Help Center',
    footerTitle: 'Resources',
    footerTagline: 'Oliver helps turn requests into finished work.',
    footerPill: 'Start personal. Expand carefully.',
    footerBottomSecondary: 'One app, one task, one reviewable result.'
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
      '认识 Oliver。它是服务工作和生活的可信 AI operator，可以从首页直接开始：连接一个工具，交给它一个真实任务，再审阅结果。',
    canonicalUrl: 'https://dowhiz.com/cn',
    ogLocale: 'zh_CN',
    themeColor: '#2C2C2E',
    htmlLang: 'zh-CN'
  },
  nav: {
    homePath: '/cn',
    links: [
      { href: '#examples', label: '实际例子' },
      { href: '#how-it-starts', label: '怎么开始' },
      { href: '#faq', label: '常见问题' }
    ],
    signIn: '登录',
    dashboard: 'Oliver 设置',
    signOut: '退出登录',
    githubAriaLabel: 'GitHub',
    discordAriaLabel: 'Discord',
    contactAriaLabel: '给 Oliver 发邮件'
  },
  hero: {
    eyebrow: '认识 Oliver',
    title: '直接告诉 Oliver 要做什么。',
    subtitle:
      'Oliver 是一个可信的 AI operator。它会在 email、Slack、Discord、GitHub、Notion 和 Lark 里工作，再把可审阅的结果带回来。',
    caption: '先从一个工具开始，第一次任务尽量小一点。',
    primaryCtaAnonymous: '从一个工具开始',
    primaryCtaAuthenticated: '继续 onboarding',
    contactSubject: '给 Oliver 的第一个任务',
    contactBody:
      '你好 Oliver，\n\n这是我想先让你帮忙处理的第一个任务：\n\n- 背景：\n- 什么算完成：\n- 截止时间：\n\n谢谢！',
    toolsEyebrow: '从一个真实入口开始',
    toolsHintAnonymous: 'Email 可以直接发。其他工具会带你进入 setup。',
    toolsHintAuthenticated: '如果你已经登录，这些按钮会直接启动真实连接流程。',
    actionLabels: {
      email: '立即发送',
      connect: '连接',
      setup: '打开 setup',
      loading: '打开中...'
    },
    tools: [
      {
        key: 'email',
        label: 'Email',
        description: '现在就可以直接发请求。',
        monogram: '@',
        accent: '#ff8a3d'
      },
      {
        key: 'slack',
        label: 'Slack',
        description: '从任务本来就在发生的聊天里开始。',
        monogram: 'S',
        accent: '#36c58b'
      },
      {
        key: 'discord',
        label: 'Discord',
        description: '在高频来回沟通里接住任务。',
        monogram: 'D',
        accent: '#6c78ff'
      },
      {
        key: 'github',
        label: 'GitHub',
        description: '连接 repo、issue 和后续动作。',
        monogram: 'GH',
        accent: '#2c2c2e'
      },
      {
        key: 'notion',
        label: 'Notion',
        description: '连接文档、笔记和任务上下文。',
        monogram: 'N',
        accent: '#6f6b63'
      },
      {
        key: 'lark',
        label: 'Lark',
        description: '连接协作和运营沟通。',
        monogram: 'L',
        accent: '#3f88ff'
      }
    ],
    flowEyebrow: 'onboarding 怎么开始',
    flowSteps: ['连接一个工具', '交给 Oliver 一个任务', '审阅结果', '把有效流程复用起来'],
    previewEyebrow: '适合先试的任务',
    previewRequest: '帮我起草这封邮件的回复，把 action items 拉进 Notion，敏感内容先保留给我确认。',
    previewResultTitle: '回来的结果',
    previewResults: ['回复草稿已准备', '待办已提取', '需要确认的部分已标出']
  },
  demo: {
    eyebrow: '真实产品演示',
    title: '先看 Oliver 怎么工作，而不是先看一大段解释。',
    intro: '一个完整桌面演示，加上三个更短的移动端例子。',
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
  story: {
    eyebrow: '怎么开始',
    title: '第一次成功应该是小而清楚，而且真的有用。',
    intro: '不是先搭一整套系统，而是先跑通一个真实闭环。',
    steps: [
      {
        id: '01',
        title: '先连接一个工具',
        description: '从任务本来就会出现的地方开始。'
      },
      {
        id: '02',
        title: '交给 Oliver 一个真实任务',
        description: '选一个具体、可复核、而且值得交出去的任务。'
      },
      {
        id: '03',
        title: '审阅后再复用',
        description: '先看结果，再保存偏好，最后再决定是否扩展。'
      }
    ],
    controlEyebrow: '审阅与控制',
    controlTitle: 'Oliver 的执行建立在你的许可上，而不是背后偷偷做事。',
    controlPoints: [
      '只连接你想让 Oliver 用的工具',
      '敏感动作前先看草稿再放行',
      '把偏好和 memory 放在一个地方维护'
    ]
  },
  examples: {
    eyebrow: '从第一天就有用',
    title: '早期最适合的任务，通常横跨工作和生活。',
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
  faqItems: [
    {
      question: '现在的 DoWhiz 是什么？',
      answer:
        'DoWhiz 现在先从 Oliver 开始。它是一个会在你的工具里工作、把结果带回来、并且可以逐步建立信任的 AI operator。'
    },
    {
      question: '第一步应该做什么？',
      answer:
        '先连接一个工具，交给 Oliver 一个真实任务，审阅结果，再把让它更好用的偏好或 memory 记下来。'
    },
    {
      question: '什么样的任务最适合先用？',
      answer:
        '研究、起草、材料整理、收件箱跟进、repo 协调，以及需要你最后确认的内容发布，都是很好的开始。'
    },
    {
      question: '我还掌控流程吗？',
      answer:
        '掌控权在你手里。你决定接哪些工具、哪些动作需要复核，以及 Oliver 什么时候才算真正值得复用。'
    },
    {
      question: 'DoWhiz 现在只做个人场景吗？',
      answer:
        '产品先从个人可用性出发。当 Oliver 已经在一个人的工作流里证明自己，再自然扩展到 SMB 的共享流程。'
    }
  ],
  labels: {
    faqEyebrow: '常见问题',
    faqTitle: '开始之前最需要知道的几件事',
    faqIntro: '只回答信任、适配度和 onboarding 相关的核心问题。',
    faqLinkLabel: '打开帮助中心',
    footerTitle: '资源',
    footerTagline: 'Oliver 帮你把请求变成结果。',
    footerPill: '先服务个人，再谨慎扩展。',
    footerBottomSecondary: '一个工具，一个任务，一个可审阅的结果。'
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
