curl -so rules.txt "$(curl -s 'https://magic.wizards.com/en/rules'|grep media.wizards.com|grep "downloads/MagicCompRules"|grep "\.txt"|sed 's/.*href="//g;s/" .*//;s/ /%20/g')"
